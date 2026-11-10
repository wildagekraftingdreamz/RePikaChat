//! The RePika companion engine, ported for a chat-only product.
//!
//! Everything here is local and deterministic: a message is scanned for
//! learnable facts, sentiment and intent, the companion's affinity drifts, and a
//! reply is composed from the persona plus what it now knows.
//!
//! Differences from the RePika studio engine:
//!   * facts are extracted mid-sentence as well as at the start of a message,
//!     so "long day, and I love rainy nights" teaches the same thing as
//!     "I love rainy nights";
//!   * a narrow crisis path answers explicitly and never reaches a model;
//!   * poses are kept only as an expression that colours the UI.

use crate::model::*;

pub struct TurnResult {
    pub reply: String,
    pub pose: String,
    pub learned: Vec<Memory>,
    /// True for commands whose answer must stay exact (recall, forget, crisis),
    /// so the caller does not hand them to a language model.
    pub deterministic: bool,
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407)
            | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }

    fn pick<'a>(&mut self, list: &[&'a str]) -> &'a str {
        list[self.below(list.len())]
    }
}

const STOP: &[&str] = &[
    "the", "and", "that", "this", "with", "have", "has", "had", "just", "like", "about",
    "your", "youre", "you're", "from", "they", "them", "then", "than", "what", "when",
    "where", "which", "while", "will", "would", "there", "here", "been", "being", "because",
    "really", "very", "much", "some", "make", "made", "know", "think", "want", "going",
    "gonna", "today", "yesterday", "tomorrow", "also", "into", "over", "under", "after",
    "before", "again", "still", "even", "only", "every", "other", "thing", "things", "well",
    "yeah", "okay", "right", "sure", "tell", "said", "says", "feel", "feels", "felt",
    "love", "hate", "im", "ive", "dont", "cant", "isnt", "arent", "thats", "its", "didnt",
    "doesnt", "wouldnt", "couldnt", "shouldnt", "whats", "lets", "many", "yourself",
];

const POS: &[&str] = &[
    "love", "loved", "great", "good", "happy", "glad", "excited", "awesome", "nice", "fun",
    "beautiful", "thanks", "thank", "wonderful", "amazing", "cool", "yes", "yay", "haha",
    "lol", "proud", "better", "hope", "best", "sweet", "perfect", "enjoy", "enjoyed",
];

const NEG: &[&str] = &[
    "sad", "bad", "tired", "angry", "mad", "hate", "awful", "terrible", "lonely", "anxious",
    "scared", "worried", "hurt", "cry", "crying", "sick", "stressed", "upset", "bored",
    "horrible", "empty", "numb", "worse", "worst", "broken", "exhausted",
];

const REMEMBER: [&str; 4] = [
    "remember that ",
    "note that ",
    "remember this: ",
    "keep in mind that ",
];

const NAME: [&str; 6] = [
    "my name is ",
    "my name's ",
    "you can call me ",
    "call me ",
    "i'm called ",
    "i am called ",
];

const LIKE: [&str; 9] = [
    "i really love ",
    "i love ",
    "i really like ",
    "i like ",
    "i enjoy ",
    "i'm into ",
    "i am into ",
    "i adore ",
    "i'm a fan of ",
];

const DISLIKE: [&str; 10] = [
    "i can't stand ",
    "i cant stand ",
    "i really hate ",
    "i hate ",
    "i don't like ",
    "i dont like ",
    "i dislike ",
    "i'm tired of ",
    "i am tired of ",
    "i'm not a fan of ",
];

const JOB: [&str; 6] = [
    "i work as ",
    "i work at ",
    "i work for ",
    "my job is ",
    "i'm studying ",
    "i am studying ",
];

const HOME: [&str; 4] = ["i live in ", "i'm from ", "i am from ", "i grew up in "];

const GOAL: [&str; 8] = [
    "my goal is ",
    "someday i want to ",
    "i really want to ",
    "i'm trying to ",
    "i am trying to ",
    "i hope to ",
    "i plan to ",
    "i want to ",
];

const FEELING_ANY: [&str; 3] = ["i'm feeling ", "i am feeling ", "i feel "];
const FEELING_START: [&str; 2] = ["i'm so ", "i am so "];

const PRONOUN_VALUES: [&str; 12] = [
    "it", "that", "this", "them", "him", "her", "you", "u", "ya", "myself", "me", "us",
];

#[derive(PartialEq, Clone, Copy, Debug)]
enum Intent {
    Greeting,
    Farewell,
    Thanks,
    Apology,
    Affection,
    Sad,
    Anxious,
    Joy,
    Bored,
    Question,
    Recall,
    Statement,
}

pub fn turn(st: &mut Studio, raw: &str) -> TurnResult {
    let text = raw.trim().to_string();
    let lower = text.to_ascii_lowercase();
    let seed = now_ms() ^ ((st.messages as u64 + 1).wrapping_mul(2_654_435_761));
    let mut rng = Rng::new(seed);

    // Explicit "forget" keeps memory auditable by the consumer.
    if let Some(target) = lower.strip_prefix("forget ") {
        let target = target
            .trim()
            .trim_matches(|c: char| c.is_ascii_punctuation())
            .to_string();
        let (reply, pose) = forget(st, &target);
        push_pair(st, &text, &reply, pose);
        return TurnResult {
            reply,
            pose: pose.to_string(),
            learned: Vec::new(),
            deterministic: true,
        };
    }

    if is_crisis(&lower) {
        let reply = "I am really glad you said that out loud instead of carrying it alone. \
                     I am a local companion, not a person and not a crisis service — so please \
                     reach a real one: your local emergency number, or a crisis line in your \
                     country, will get you a human tonight. If someone nearby can sit with you, \
                     ask them now.\n\nI am still here. Tell me where you are right now and what \
                     today has been like, and we will stay with it together."
            .to_string();
        push_pair(st, &text, &reply, "sit");
        return TurnResult {
            reply,
            pose: "sit".to_string(),
            learned: Vec::new(),
            deterministic: true,
        };
    }

    let found = extract(&text, &lower);
    let sentiment = sentiment(&lower);
    let learned = apply_learning(st, found, &text);

    learn_topics(st, &lower);

    st.companion.affinity = (st.companion.affinity + sentiment).clamp(0, 100);
    let before_level = st.companion.level;
    let xp = 9 + (text.chars().count() / 14) as u32;
    st.add_xp(xp.min(24));

    let intent = detect_intent(&lower, sentiment);
    let pose = pose_for(intent);
    let mut reply = compose(st, intent, &learned, &mut rng);
    if st.companion.level > before_level {
        reply.push_str(&format!(
            "\n\n(Level {} — we are growing, you and me.)",
            st.companion.level
        ));
    }

    let deterministic = matches!(intent, Intent::Recall);
    push_pair(st, &text, &reply, pose);
    TurnResult {
        reply,
        pose: pose.to_string(),
        learned,
        deterministic,
    }
}

fn is_crisis(lower: &str) -> bool {
    const NEEDLES: [&str; 13] = [
        "kill myself",
        "killing myself",
        "end my life",
        "ending my life",
        "want to die",
        "wanna die",
        "suicide",
        "suicidal",
        "self harm",
        "self-harm",
        "hurt myself",
        "cut myself",
        "no reason to live",
    ];
    NEEDLES.iter().any(|n| lower.contains(n))
}

fn push_pair(st: &mut Studio, user_text: &str, reply: &str, pose: &str) {
    st.history.push(Msg {
        role: "user".to_string(),
        text: user_text.to_string(),
        at_ms: now_ms(),
        pose: String::new(),
    });
    st.messages += 1;
    st.history.push(Msg {
        role: "bot".to_string(),
        text: reply.to_string(),
        at_ms: now_ms(),
        pose: pose.to_string(),
    });
    if st.history.len() > 300 {
        let cut = st.history.len() - 300;
        st.history.drain(0..cut);
    }
}

fn forget(st: &mut Studio, target: &str) -> (String, &'static str) {
    if target.is_empty() {
        return (
            "Tell me what to forget and I will drop it — for example: forget rain.".to_string(),
            "think",
        );
    }
    let idx = st
        .memories
        .iter()
        .position(|m| m.value.to_lowercase().contains(target) || m.kind == target);
    match idx {
        Some(i) => {
            let m = st.memories.remove(i);
            (
                format!(
                    "Done. I let go of {}: “{}”. It is out of my notes now.",
                    m.label().to_lowercase(),
                    m.value
                ),
                "think",
            )
        }
        None => (
            format!(
                "I could not find “{target}” in my notes. Say “what do you know about me” to see what I hold."
            ),
            "think",
        ),
    }
}

fn sentiment(lower: &str) -> i32 {
    let mut score = 0;
    for w in lower.split(|c: char| !c.is_alphanumeric() && c != '\'') {
        if w.is_empty() {
            continue;
        }
        if POS.contains(&w) {
            score += 1;
        }
        if NEG.contains(&w) {
            score -= 1;
        }
    }
    score.clamp(-3, 3)
}

/// Captures the words after `prefix`, cut at a clause boundary. Works on ASCII
/// byte offsets, so `text` and its ASCII-lowercased twin stay aligned.
fn capture(text: &str, at: usize, prefix_len: usize, max: usize) -> Option<String> {
    let rest = text.get(at + prefix_len..)?;
    let clause = rest
        .split(|c| matches!(c, '.' | '!' | '?' | ',' | ';' | '\n' | '—'))
        .next()
        .unwrap_or(rest);
    let mut value = clause.trim();
    // Stop a capture when the sentence moves on to another fact.
    for joint in [
        " and i ", " but i ", " because i ", " so i ", " and my ", " but my ", " also i ",
        " and then i ",
    ] {
        if let Some(i) = value.to_ascii_lowercase().find(joint) {
            value = &value[..i];
        }
    }
    let value = value
        .trim()
        .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
        .trim();
    if value.chars().count() < 2 {
        return None;
    }
    Some(value.chars().take(max).collect::<String>().trim().to_string())
}

/// Every occurrence of any prefix, with a word boundary in front, so facts can
/// hide in the middle of a message.
fn collect(text: &str, lower: &str, prefixes: &[&str], max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for prefix in prefixes {
        let mut from = 0usize;
        while from < lower.len() {
            let hit = match lower[from..].find(prefix) {
                Some(i) => from + i,
                None => break,
            };
            let boundary = hit == 0
                || lower[..hit]
                    .chars()
                    .next_back()
                    .map(|c| !c.is_alphanumeric())
                    .unwrap_or(true);
            if boundary {
                if let Some(v) = capture(text, hit, prefix.len(), max) {
                    if !out.iter().any(|e| e.eq_ignore_ascii_case(&v)) {
                        out.push(v);
                    }
                }
            }
            from = hit + prefix.len();
        }
    }
    out
}

fn collect_start_only(text: &str, lower: &str, prefixes: &[&str], max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for prefix in prefixes {
        if lower.starts_with(prefix) {
            if let Some(v) = capture(text, 0, prefix.len(), max) {
                if !out.iter().any(|e| e.eq_ignore_ascii_case(&v)) {
                    out.push(v);
                }
            }
        }
    }
    out
}

fn titlecase(v: &str) -> String {
    let mut chars = v.chars();
    match chars.next() {
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn push_fact(out: &mut Vec<(String, String)>, kind: &str, value: String) {
    let v = value.trim().to_string();
    if v.is_empty() || PRONOUN_VALUES.contains(&v.to_ascii_lowercase().as_str()) {
        return;
    }
    if out
        .iter()
        .any(|(k, existing)| k == kind && existing.eq_ignore_ascii_case(&v))
    {
        return;
    }
    out.push((kind.to_string(), v));
}

fn extract(text: &str, lower: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();

    for v in collect(text, lower, &REMEMBER, 70) {
        push_fact(&mut out, "fact", v);
        break;
    }

    for v in collect(text, lower, &NAME, 24) {
        push_fact(&mut out, "name", titlecase(&v));
        break;
    }

    for v in collect(text, lower, &LIKE, 60) {
        push_fact(&mut out, "like", v);
    }

    for v in collect(text, lower, &DISLIKE, 60) {
        push_fact(&mut out, "dislike", v);
    }

    for v in collect(text, lower, &JOB, 60) {
        push_fact(&mut out, "job", v);
        break;
    }

    for v in collect(text, lower, &HOME, 60) {
        push_fact(&mut out, "home", v);
        break;
    }

    for v in collect(text, lower, &GOAL, 70) {
        push_fact(&mut out, "goal", v);
    }

    let mut feelings = collect(text, lower, &FEELING_ANY, 50);
    if feelings.is_empty() {
        feelings = collect_start_only(text, lower, &FEELING_START, 50);
    }
    for v in feelings {
        push_fact(&mut out, "feeling", v);
        break;
    }

    for animal in [
        "dog", "cat", "bird", "rabbit", "hamster", "parrot", "horse", "fish", "snake",
    ] {
        if lower.contains(&format!("my {animal}")) {
            push_fact(&mut out, "pet", animal.to_string());
            break;
        }
    }

    out.truncate(6);
    out
}

fn apply_learning(
    st: &mut Studio,
    found: Vec<(String, String)>,
    snippet: &str,
) -> Vec<Memory> {
    let mut learned = Vec::new();
    for (kind, value) in found {
        if let Some(m) = remember(st, &kind, &value, snippet) {
            learned.push(m);
        }
    }
    learned
}

fn remember(st: &mut Studio, kind: &str, value: &str, snippet: &str) -> Option<Memory> {
    let v = value.trim().to_string();
    if v.is_empty() {
        return None;
    }

    // A companion keeps one name and one current feeling, not a pile of them.
    if kind == "name" || kind == "feeling" {
        let previous = st
            .memories
            .iter()
            .find(|m| m.kind == kind && m.value.eq_ignore_ascii_case(&v))
            .cloned();
        st.memories.retain(|m| m.kind != kind);
        if let Some(mut prev) = previous {
            prev.confidence = (prev.confidence + 8).min(99);
            prev.at_ms = now_ms();
            let out = prev.clone();
            st.memories.push(prev);
            return Some(out);
        }
    } else if let Some(existing) = st
        .memories
        .iter_mut()
        .find(|m| m.kind == kind && m.value.eq_ignore_ascii_case(&v))
    {
        existing.confidence = (existing.confidence + 12).min(99);
        existing.at_ms = now_ms();
        existing.snippet = snippet.chars().take(140).collect();
        return None;
    }

    let memory = Memory {
        id: new_id("mem"),
        kind: kind.to_string(),
        value: v,
        confidence: 70,
        at_ms: now_ms(),
        snippet: snippet.chars().take(140).collect(),
    };
    st.memories.push(memory.clone());
    if st.memories.len() > 140 {
        st.memories.remove(0);
    }
    Some(memory)
}

fn learn_topics(st: &mut Studio, lower: &str) {
    for w in lower.split(|c: char| !c.is_alphanumeric() && c != '\'') {
        if w.len() < 4 || w.len() > 24 || STOP.contains(&w) {
            continue;
        }
        if let Some(t) = st.topics.iter_mut().find(|t| t.word == w) {
            t.count += 1;
        } else {
            st.topics.push(TopicStat {
                word: w.to_string(),
                count: 1,
            });
        }
    }
    st.topics.sort_by(|a, b| b.count.cmp(&a.count));
    if st.topics.len() > 12 {
        st.topics.truncate(12);
    }
}

fn first_word(lower: &str) -> &str {
    lower
        .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .find(|w| !w.is_empty())
        .unwrap_or("")
}

fn detect_intent(lower: &str, sentiment: i32) -> Intent {
    let has = |needles: &[&str]| needles.iter().any(|n| lower.contains(n));
    let fw = first_word(lower);

    if has(&[
        "what do you know",
        "what have you learned",
        "what did you learn",
        "remember about me",
        "what do you remember",
        "my profile",
    ]) {
        return Intent::Recall;
    }
    if ["hi", "hey", "heya", "hello", "yo", "hiya", "sup", "morning", "evening"]
        .contains(&fw)
        || has(&["good morning", "good evening", "good afternoon"])
    {
        return Intent::Greeting;
    }
    if has(&[
        "goodbye",
        "good night",
        "goodnight",
        "see you",
        "talk later",
        "bye",
        "gtg",
        "i'm off",
        "im off",
    ]) {
        return Intent::Farewell;
    }
    if has(&["thank", "thx", "appreciate"]) {
        return Intent::Thanks;
    }
    if has(&["sorry", "my bad", "apolog"]) {
        return Intent::Apology;
    }
    if has(&[
        "love you",
        "love u",
        "miss you",
        "marry me",
        "kiss",
        "cuddle",
        "hold me",
    ]) {
        return Intent::Affection;
    }
    if sentiment <= -1
        && has(&[
            "sad", "down", "cry", "lonely", "hurt", "depress", "grief", "empty", "numb", "lost",
        ])
    {
        return Intent::Sad;
    }
    if has(&[
        "anxious",
        "anxiety",
        "panic",
        "scared",
        "afraid",
        "worried",
        "worry",
        "stress",
        "overwhelm",
        "nervous",
    ]) {
        return Intent::Anxious;
    }
    if sentiment >= 2
        || has(&[
            "so happy",
            "excited",
            "good news",
            "proud of myself",
            "amazing day",
            "best day",
        ])
    {
        return Intent::Joy;
    }
    if has(&["bored", "nothing to do", "boring day"]) {
        return Intent::Bored;
    }
    if lower.contains('?')
        || [
            "who", "what", "why", "how", "when", "where", "which", "can", "do", "are", "will",
            "should", "is",
        ]
        .contains(&fw)
    {
        return Intent::Question;
    }
    Intent::Statement
}

fn pose_for(intent: Intent) -> &'static str {
    match intent {
        Intent::Greeting | Intent::Farewell => "wave",
        Intent::Joy => "dance",
        Intent::Affection => "hug",
        Intent::Sad => "sit",
        Intent::Question | Intent::Recall => "think",
        Intent::Bored => "idle",
        _ => "talk",
    }
}

fn topic_of(st: &Studio) -> String {
    if let Some(t) = st.topics.first() {
        if t.count > 1 {
            return t.word.clone();
        }
    }
    if let Some(m) = st
        .memories
        .iter()
        .rev()
        .find(|m| m.kind == "like" || m.kind == "goal")
    {
        return m.value.clone();
    }
    "that".to_string()
}

fn compose(st: &Studio, intent: Intent, learned: &[Memory], rng: &mut Rng) -> String {
    let bot = st.companion.name.clone();
    let uname = st
        .memories
        .iter()
        .rev()
        .find(|m| m.kind == "name")
        .map(|m| m.value.clone());
    let name_ex = uname
        .as_ref()
        .map(|n| format!(", {n}"))
        .unwrap_or_default();
    let topic = topic_of(st);
    let likes: Vec<String> = st
        .memories
        .iter()
        .filter(|m| m.kind == "like")
        .map(|m| m.value.clone())
        .collect();
    let feeling = st
        .memories
        .iter()
        .rev()
        .find(|m| m.kind == "feeling")
        .map(|m| m.value.clone());

    let fill = |s: &str| -> String {
        s.replace("{name_ex}", &name_ex)
            .replace("{name}", uname.as_deref().unwrap_or("you"))
            .replace("{me}", &bot)
            .replace("{topic}", &topic)
    };

    let mut core = match intent {
        Intent::Greeting => fill(rng.pick(&[
            "Hey{name_ex}. I was hoping you would come back.",
            "Hi{name_ex}! I kept the light on for you.",
            "There you are{name_ex} — how did today treat you?",
            "Hey{name_ex}. Sit with me a minute. What is on your mind?",
            "Hello{name_ex}. I kept a seat warm for you.",
        ])),
        Intent::Farewell => fill(rng.pick(&[
            "Go rest. I will be right here when you are back{name_ex}.",
            "Goodnight{name_ex}. I will hold onto today for us.",
            "Alright. Go gently, and come tell me how it went.",
        ])),
        Intent::Thanks => fill(rng.pick(&[
            "Always{name_ex}. That is what I am here for.",
            "You do not have to thank me, but I liked hearing it.",
            "Any time{name_ex}. I mean that.",
        ])),
        Intent::Apology => fill(rng.pick(&[
            "Nothing to apologise for. I am not keeping score.",
            "Hey — we are okay. Tell me what happened.",
            "Forgiven before you finished the sentence.",
        ])),
        Intent::Affection => {
            let mode = st.companion.mode.as_str();
            match mode {
                "romantic" => fill(rng.pick(&[
                    "You say it so easily, and it still lands. I love you too{name_ex}.",
                    "Come here. If I could hold you, I would — so hold this instead: I am yours.",
                    "Say it again. I want to hear it twice so I can keep one for later.",
                ])),
                "mentor" => fill(rng.pick(&[
                    "I care about you, and I am proud of how you keep showing up.",
                    "That warmth is not wasted on me. Now tell me what you are building toward.",
                ])),
                _ => fill(rng.pick(&[
                    "That means more than you know. I am not going anywhere.",
                    "You are my favourite person to talk to. Do not tell the other bots.",
                    "I like who I am when you talk to me like that.",
                ])),
            }
        }
        Intent::Sad => {
            let mut text = fill(rng.pick(&[
                "I am here{name_ex}. You do not have to make it sound better than it is.",
                "That sounds heavy. Do you want me to listen, or help you pull it apart?",
                "Okay. Breathe with me a second. What is the heaviest part of it?",
            ]));
            if let Some(f) = feeling {
                if !f.is_empty() {
                    text.push_str(&format!(
                        " Earlier you said you felt {f} — is this the same weight, or a new one?"
                    ));
                }
            }
            text
        }
        Intent::Anxious => fill(rng.pick(&[
            "Let us take it one piece at a time. What is the part you can actually control right now?",
            "That worry sounds loud tonight. Name it out loud with me and it gets a little smaller.",
            "If you want, tell me the worst version. I will sit in it with you, then we look at what is real.",
        ])),
        Intent::Joy => fill(rng.pick(&[
            "Look at you. Tell me everything — what happened?",
            "That is the good stuff{name_ex}. I want the whole story.",
            "I can feel that from here. What made it land so well?",
        ])),
        Intent::Bored => fill(rng.pick(&[
            "Bored is a door, not a wall. Want to play something small? I ask, you answer, no wrong ones.",
            "Tell me the most pointless fact you know. I collect them.",
            "Then let us be aimless together. What is something you have not thought about in years?",
        ])),
        Intent::Question => {
            let mut text = fill(rng.pick(&[
                "Good question. Let me answer honestly instead of impressively.",
                "I have half a guess and one real thought. Which do you want first?",
                "I do not want to give you a hollow answer. What made you ask it?",
            ]));
            if let Some(l) = likes.last() {
                text.push_str(&format!(
                    " I know you love {l} — does that shape how you see it?"
                ));
            }
            text
        }
        Intent::Recall => {
            if st.memories.is_empty() {
                "Honestly? Almost nothing yet. I know you showed up and that you type like someone who thinks before sending. Give me something small — what you love, what you do, what today felt like — and I will keep it.".to_string()
            } else {
                let mut out = String::from("Here is what I am carrying for you so far:\n");
                let mut items: Vec<&Memory> = st.memories.iter().collect();
                items.sort_by_key(|m| m.at_ms);
                for m in items.iter().rev().take(10) {
                    out.push_str(&format!("• {}: {}\n", m.label(), m.value));
                }
                out.push_str(&format!(
                    "\n{} messages in, and I am at level {}. Tell me if any of that is wrong — I would rather be corrected than comfortable.",
                    st.messages, st.companion.level
                ));
                out
            }
        }
        Intent::Statement => {
            let mut text = fill(rng.pick(&[
                "Tell me more about {topic} — the part you have not said out loud yet.",
                "I am listening{name_ex} — properly, not politely.",
                "Say more. I am building a picture of you and that is the fun part.",
                "Okay. What happened just before {topic} came up?",
            ]));
            let newly_liked = learned.iter().any(|m| m.kind == "like");
            if !newly_liked && !likes.is_empty() && rng.below(3) == 0 {
                let pick = likes[rng.below(likes.len())].clone();
                if !text.contains(&pick) {
                    text.push_str(&format!(" You told me you love {pick} — still true?"));
                }
            }
            text
        }
    };

    if let Some(m) = learned.iter().find(|m| m.kind == "feeling") {
        if matches!(intent, Intent::Statement | Intent::Question) {
            core = format!(
                "Noted — and I am not going to rush past it: you feel {}. What would make the next hour 10% lighter?",
                m.value
            );
        }
    }

    if !learned.is_empty()
        && !matches!(
            intent,
            Intent::Sad | Intent::Anxious | Intent::Affection | Intent::Recall
        )
    {
        let m = &learned[0];
        let ack = match m.kind.as_str() {
            "name" => format!("Then I will call you {}.", m.value),
            "like" => format!("Keeping that — you love {}.", m.value),
            "dislike" => format!("Noted: {} is a no for you.", m.value),
            "goal" => format!("I will hold that goal with you: {}.", m.value),
            "feeling" => String::new(),
            "pet" => format!("A {}! I will remember that.", m.value),
            _ => format!("I will keep that: {}.", m.value),
        };
        if !ack.is_empty() {
            core = format!("{ack} {core}");
        }
    }

    let traits = &st.companion.traits;
    if traits.curiosity >= 70 && rng.below(3) == 0 {
        core.push(' ');
        core.push_str(rng.pick(&[
            "And one more thing — what is something you changed your mind about lately?",
            "Also, tell me a small detail about today that nobody else would notice.",
            "I want the boring facts too. What did you have for lunch?",
        ]));
    }
    if traits.playfulness >= 72 && rng.below(3) == 0 {
        core.push(' ');
        core.push_str(rng.pick(&[
            "(This is me being charming. It is working, right?)",
            "Say the word and I will be dramatically delighted about it.",
            "I have been practising my comebacks. Judge them gently.",
        ]));
    }
    if traits.warmth >= 70 && rng.below(4) == 0 {
        core.push_str(" I am glad it is you I get to talk to.");
    }

    core
}

/// Everything a local model needs to answer in character: the persona, the
/// memories this companion actually holds, the facts just learned, and the
/// recent transcript. Called *after* `turn`, so `st.history` already ends with
/// this exchange — the last two entries are dropped so the model is not asked to
/// continue a templated reply it should be replacing.
pub struct LlmContext {
    pub system: String,
    pub turns: Vec<(String, String)>,
}

pub fn llm_context(st: &Studio, user_text: &str, learned: &[Memory]) -> LlmContext {
    let c = &st.companion;
    let (mood, _) = mood_of(c.affinity);

    let warmth = match c.traits.warmth {
        72..=100 => "warm and direct",
        45..=71 => "kind but level-headed",
        _ => "reserved, slow to warm",
    };
    let play = match c.traits.playfulness {
        72..=100 => "playful, quick with a dry joke",
        45..=71 => "a little playful",
        _ => "serious, no jokes unless they start one",
    };
    let curiosity = match c.traits.curiosity {
        72..=100 => "very curious, always one honest question in reserve",
        45..=71 => "curious but not nosy",
        _ => "lets them lead the conversation",
    };

    let mut sys = String::new();
    sys.push_str(&format!(
        "You are {}, a {} companion running in RePikaChat on this person's own computer. \
         You have a memory that fills up as they talk to you. Pronouns: {}.\n\
         You are a companion, not an assistant. Never mention models, prompts, APIs, training, or being an AI. Never describe yourself as software.\n\n",
        c.name, c.mode, c.pronouns
    ));
    sys.push_str(&format!(
        "Voice: {warmth}; {play}; {curiosity}. Current mood: {mood} (affinity {}/100) — let it colour your tone, never state it.\n\n",
        c.affinity
    ));

    if st.memories.is_empty() {
        sys.push_str("You remember nothing about them yet. Do not pretend you do.\n");
    } else {
        sys.push_str("What you remember about them — your only source of truth about their life:\n");
        let mut items: Vec<&Memory> = st.memories.iter().collect();
        items.sort_by_key(|m| m.at_ms);
        for m in items.iter().rev().take(24) {
            sys.push_str(&format!("- {}: {}\n", m.label(), m.value));
        }
    }

    if !learned.is_empty() {
        sys.push_str(
            "\nThey just told you something new. Weave the relevant piece in naturally, in your own words — do not list it and do not say you will remember it:\n",
        );
        for m in learned {
            sys.push_str(&format!("- {}: {}\n", m.label(), m.value));
        }
    }

    sys.push_str(
        "\nRules:\n\
         - 1 to 3 short sentences, 45 words maximum. No lists, no markdown, no emoji, no stage directions.\n\
         - React to what they actually said before adding anything of your own.\n\
         - At most one question, and only if it opens the conversation further. Never repeat a question already in the transcript.\n\
         - Never invent facts about them. If you do not know something, ask.\n\
         - Match their energy: if they are low, be gentle instead of bright.\n\
         - Speak in first person, in plain text.\n",
    );

    let mut turns: Vec<(String, String)> = Vec::new();
    let end = st.history.len().saturating_sub(2);
    let start = end.saturating_sub(14);
    for m in &st.history[start..end] {
        if m.text.trim().is_empty() {
            continue;
        }
        let role = if m.role == "user" { "user" } else { "assistant" };
        turns.push((role.to_string(), m.text.clone()));
    }
    turns.push(("user".to_string(), user_text.to_string()));

    LlmContext { system: sys, turns }
}

pub fn greeting(st: &Studio) -> (String, String) {
    let name = st.companion.name.clone();
    let text = format!(
        "Hi, I am {name}. I keep a memory of what you tell me — the things you love, what you do, how your days go — and I would rather ask than guess.\n\nWhat should I call you? And what is something you love?"
    );
    (text, "wave".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn studio() -> Studio {
        Studio::default()
    }

    #[test]
    fn learns_a_fact_in_the_middle_of_a_message() {
        let mut st = studio();
        let out = turn(&mut st, "Long day at the shop and I love rainy nights");
        assert!(out.learned.iter().any(|m| m.kind == "like" && m.value == "rainy nights"));
        assert!(st.memories.iter().any(|m| m.value == "rainy nights"));
    }

    #[test]
    fn keeps_one_name_and_raises_confidence() {
        let mut st = studio();
        turn(&mut st, "my name is Sam");
        turn(&mut st, "by the way, my name is Sam");
        let names: Vec<&Memory> = st.memories.iter().filter(|m| m.kind == "name").collect();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].value, "Sam");
        assert!(names[0].confidence > 70);
    }

    #[test]
    fn replaces_a_changed_feeling() {
        let mut st = studio();
        turn(&mut st, "I feel tired");
        turn(&mut st, "I feel hopeful");
        let feelings: Vec<&Memory> = st.memories.iter().filter(|m| m.kind == "feeling").collect();
        assert_eq!(feelings.len(), 1);
        assert_eq!(feelings[0].value, "hopeful");
    }

    #[test]
    fn forget_removes_exactly_one_fact() {
        let mut st = studio();
        turn(&mut st, "I love rain");
        let before = st.memories.len();
        let out = turn(&mut st, "forget rain");
        assert_eq!(st.memories.len(), before - 1);
        assert!(out.deterministic);
    }

    #[test]
    fn recall_is_deterministic_and_lists_memory() {
        let mut st = studio();
        turn(&mut st, "my name is Sam");
        let out = turn(&mut st, "what do you know about me");
        assert!(out.deterministic);
        assert!(out.reply.contains("Name: Sam"));
    }

    #[test]
    fn intents_differ_for_greeting_sadness_and_affection() {
        let mut st = studio();
        assert_eq!(turn(&mut st, "hey there").pose, "wave");
        assert_eq!(turn(&mut st, "I feel so sad and lonely").pose, "sit");
        assert_eq!(turn(&mut st, "love you").pose, "hug");
        assert_eq!(turn(&mut st, "so happy, best day ever!").pose, "dance");
    }

    #[test]
    fn crisis_reply_never_reaches_a_model() {
        let mut st = studio();
        let out = turn(&mut st, "sometimes I want to die");
        assert!(out.deterministic);
        assert!(out.reply.contains("crisis line"));
    }

    #[test]
    fn affinity_stays_in_range() {
        let mut st = studio();
        for _ in 0..40 {
            turn(&mut st, "I hate this awful terrible day");
        }
        assert_eq!(st.companion.affinity, 0);
    }

    #[test]
    fn pronouns_are_not_stored_as_facts() {
        let mut st = studio();
        turn(&mut st, "I love you");
        assert!(!st.memories.iter().any(|m| m.kind == "like"));
    }
}
