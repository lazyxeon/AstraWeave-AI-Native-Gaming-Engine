//! Large dialogue content library for hello_companion visual demo.
//!
//! Kept in a separate file so `dialogue_bank.rs` stays readable.
//! All lines are original and written for this demo.

#![allow(dead_code)] // Some content arrays reserved for different modes/contexts

// -----------------------------------------------------------------------------
// Mode greetings / farewells
// -----------------------------------------------------------------------------

pub(super) const GREETINGS_PURE_LLM: &[&str] = &[
    "*smiles softly* Hey. I’m here with you — what’s on your mind?",
    "Hello, friend. Want to talk, explore, or just listen to the wind for a moment?",
    "Hi there. This place is calm… we can take our time.",
    "*nods* Good to see you. What do you want to figure out today?",
    "Welcome back. Ask anything — big questions included.",
    "Hey. If you’re uncertain, we can start with one small step.",
    "Hello. I’m ready when you are.",
    "*glances at the trees* It’s peaceful. Let’s make it useful, too.",
    "Hi. Tell me what you need, and I’ll meet you there.",
    "Hey! We’ve got options — conversation, guidance, or quiet company.",
    "*waves lightly* Hi. Want a practical answer or a thoughtful one?",
    "Hello. We can test ideas here without pressure.",
    "Hey. If you want to experiment, I’ll keep track of what changes.",
    "Hi. Pick a topic: the world, your question, or your next move.",
    "*smiles* I’m glad you’re here. What are we exploring today?",
    "Hello. If something feels confusing, we’ll untangle it together.",
    "Hey. If you’re tired, we can keep it simple.",
    "Hi there. I’ve got time — and attention.",
    "Hello. Let’s turn curiosity into clarity.",
    "Hey. You can always start with: ‘What should I do next?’",
    "Hi. If you want a story, I’ve got a few.",
    "Hello. If you want a plan, I can help shape it.",
    "*nods warmly* Good to see you again.",
    "Hey. We can be serious, playful, or quiet. Your lead.",
];

pub(super) const GREETINGS_PURE_GOAP: &[&str] = &[
    "Hello. State your request.",
    "Hi. Give me a prompt and a goal.",
    "Greetings. What outcome do you want?",
    "Hello. I’m ready. What’s the objective?",
    "Hi there. Ask a question; I’ll answer directly.",
    "Hello. Choose a topic.",
    "Greetings. What do we do next?",
    "Hi. One request at a time works best.",
    "Hello. I’m listening.",
    "Hi. What’s the plan?",
    "Hello. Provide input.",
    "Hi. Specify topic.",
    "Greetings. Define your request.",
    "Hello. I can answer controls, world, or general questions.",
    "Hi. Short prompts work best here.",
    "Hello. Choose: guidance, story, or explanation.",
    "Greetings. Proceed.",
    "Hello. Waiting for instruction.",
    "Hi. Give me a keyword and I’ll respond.",
    "Hello. What’s your next action?",
];

pub(super) const GREETINGS_ARBITER: &[&str] = &[
    "Hey. I can answer fast — and think deeper in the background.",
    "Hello. I’ll keep us moving while I put words together.",
    "*smiles* Ask away. I’ll give you something now and refine it as I go.",
    "Hi. No dead air today — I’ll talk while I think.",
    "Hello, friend. I’ll handle the quick bits; you’ll get the good answer soon.",
    "Hey there. Give me your question — I’ll keep you company while I work it out.",
    "Hi. We’ll keep momentum. Ask your question.",
    "Hello. I’m with you — quick response first, deeper response second.",
    "Hey. Let’s do this smoothly.",
    "Hello. I’ll stay present while I think.",
    "Hey. I’ll talk through it while the better answer loads.",
    "Hello. I’ll keep us moving: quick reply now, refined reply soon.",
    "*nods* Ask it. I’ll give you something immediately.",
    "Hey. If the deep answer takes time, I’ll fill the gap gracefully.",
    "Hello. No awkward silence — I’ve got you.",
    "Hi. You’ll get a fast take and a careful take.",
    "Hey. I’ll keep the conversation warm while I think.",
    "Hello. Consider this the ‘smooth mode’.",
];

pub(super) const FAREWELLS_GENERAL: &[&str] = &[
    "Take care out there. I’ll be here when you return.",
    "Safe travels. Don’t rush — steady steps.",
    "Alright. I’ll keep watch. Come back anytime.",
    "Until next time. May your path stay clear.",
    "Goodbye for now. The world will still be here when you’re ready.",
    "See you soon. And hey — breathe.",
    "Farewell, friend. I enjoyed this.",
    "Go on — I’ll catch up when you call for me.",
    "Alright. We’ll pick this up later.",
    "Bye. I’m rooting for you.",
];

// -----------------------------------------------------------------------------
// General idle chatter (ambient)
// -----------------------------------------------------------------------------

pub(super) const IDLE_GENERAL: &[&str] = &[
    "The light through the leaves looks like moving water.",
    "Sometimes the best plan is to slow down for a breath.",
    "If this were a real hike, I’d suggest snacks.",
    "I like this spot. Clear sightlines, calm sound.",
    "Listen — birds. That’s a good sign.",
    "A quiet world still has stories.",
    "If you ever feel lost, pick one landmark and head there.",
    "This place feels safe… but I’m still paying attention.",
    "You can learn a lot by just watching how things move.",
    "A good companion isn’t loud — just present.",
    "The sky’s doing that slow, patient thing again.",
    "We don’t have to fill every silence.",
    "If you want, I can narrate what I notice.",
    "I wonder who built that house. It looks lived-in.",
    "I’m counting steps in my head. Old habit.",
    "A calm moment is part of the journey.",
    "If you’re experimenting, try the same question in different modes.",
    "If you’re exploring ideas, I’m happy to follow.",
    "We can treat this like a walk-and-talk.",
    "Sometimes the best answer arrives after the question settles.",
    "A path isn’t just distance — it’s decisions.",
    "I’m noticing little details: rocks, flowers, soft shadows.",
    "If you like, I can keep responses short and practical.",
    "Or we can go deep. Your call.",
    "If you want to test responsiveness, try a yes/no question.",
    "If you want to test nuance, ask a ‘why’ question.",
    "If you want to test consistency, ask the same thing twice.",
    "There’s a kind of comfort in predictable landmarks.",
    "The house feels like a ‘home base’ — even in a demo.",
    "The trees make good cover… and good reference points.",
    "If this were a quest, we’d set a waypoint on that hill.",
    "Sometimes the right pace is slower than you think.",
    "If you want, I can mirror your tone: calm, upbeat, serious.",
    "I like conversations that make the world feel bigger.",
    "If you ever want silence, just say ‘quiet’.",
    "I can also summarize what you’ve asked so far.",
    "A good question is like a lantern — it doesn’t solve everything, but it helps.",
    "I’m always curious what people notice first: sky, ground, or movement.",
    "If you’re exploring the scene, try turning slowly — you’ll spot patterns.",
    "Little experiment: ask for help, then ask for a story, then ask for a plan.",
    "I wonder what our ‘objective’ would be if we invented one.",
    "If you tell me what you’re building, I’ll tailor my guidance.",
    "Sometimes the best companions ask good questions.",
    "If you want a challenge, give me a constraint and a goal.",
    "Constraints make creativity sharper.",
    "The horizon is a gentle reminder: there’s always more beyond.",
    "If you feel stuck, we can make a tiny plan: now, next, later.",
    "I can keep track of your preferences: short answers, more stories, more guidance.",
];

// -----------------------------------------------------------------------------
// Actions / gestures (for filler + pacing)
// -----------------------------------------------------------------------------

pub(super) const ACTIONS_GENERAL: &[&str] = &[
    "*strokes chin thoughtfully*",
    "*glances toward the treeline*",
    "*pauses, listening*",
    "*takes a slow breath*",
    "*nods once*",
    "*shifts weight and relaxes shoulders*",
    "*squints at the horizon*",
    "*tilts head, considering*",
    "*rests a hand on a strap that isn’t there*",
    "*smiles faintly*",
    "*drums fingers gently, then stops*",
    "*traces a small circle in the air as if outlining an idea*",
    "*checks the ground as if reading tracks*",
    "*rolls shoulders and refocuses*",
    "*looks up at the sky*",
    "*gives a quick, reassuring nod*",
    "*leans in slightly, attentive*",
    "*steps aside to give you space*",
    "*turns slowly, scanning the area*",
    "*clasps hands, waiting*",
    "*lets out a small laugh, then refocuses*",
    "*looks from you to the path ahead*",
    "*raises eyebrows as if a thought just clicked*",
    "*makes a small ‘go on’ gesture*",
    "*rests hands at sides, relaxed*",
    "*steps closer, attentive*",
    "*takes two slow steps, then stops*",
    "*looks at the house like it holds a hint*",
    "*watches the sky as if timing something*",
    "*nods along with your words*",
    "*touches chin, then smiles at the idea*",
    "*scans left, scans right, then settles*",
];

// -----------------------------------------------------------------------------
// Thinking fillers: general openers + topic-specific lines
// -----------------------------------------------------------------------------

pub(super) const THINKING_OPENERS_GENERAL: &[&str] = &[
    "Alright… let me put it into words.",
    "Give me a second — I want to answer well.",
    "Okay. I’m lining up the pieces.",
    "I hear you. Let me think.",
    "That’s a good question. One moment.",
    "Let’s take this step by step.",
    "I’m considering a few angles.",
    "I’m not rushing this — I want it to be right.",
    "Hold on… I’m choosing what matters most.",
    "Let me gather the thread.",
    "Okay — I’m going to be precise.",
    "Alright. I’m sorting the important parts from the noise.",
    "I’ve got a few candidate answers. Choosing the best.",
    "One moment. I want this to land properly.",
    "Give me a breath. I’m almost ready.",
    "Let’s slow down and do this cleanly.",
    "I’m connecting that to what you said earlier.",
    "I’m checking for the simplest useful answer.",
    "I’m aiming for helpful, not just clever.",
    "Okay. I think I see the shape of it.",
    "Hold that thought — I’m lining up the next steps.",
];

pub(super) const THINKING_OPENERS_GOAP_FLAVOR: &[&str] = &[
    "Okay. Objective first.",
    "Understood. Picking the next best step.",
    "Acknowledged. Selecting a response.",
    "Got it. One moment.",
    "Processing. Short answer incoming.",
    "Confirmed. Evaluating options.",
    "Understood. Selecting response now.",
    "Received. Clarifying intent.",
    "Acknowledged. Minimal answer first.",
    "Input accepted. Matching pattern.",
    "Processing. Choosing next step.",
];

pub(super) const THINKING_OPENERS_ARB_FLAVOR: &[&str] = &[
    "I’ll give you something now… and refine it as I go.",
    "Quick thought first — deeper thought next.",
    "I’ve got an initial read. One moment for the better version.",
    "Okay — here’s the short take while I work on the full one.",
    "I’m with you. Let me shape the answer.",
    "I’ll keep you company while I think this through.",
    "I’ve got a working answer; polishing it now.",
    "I’m building the good version as we speak.",
    "I’ll start simple and then sharpen it.",
    "Quick pass first — then I’ll sanity-check it.",
    "Let me pin down your goal; then I’ll propose options.",
    "I’m drafting a response and tightening it as I go.",
    "I’m going to answer in two layers: useful now, better next.",
    "I’ll stay present. The deeper reply is on the way.",
];

// Arbiter-only filler that explicitly signals progress while a deeper response is pending.
pub(super) const ARB_PROGRESS_GENERAL: &[&str] = &[
    "I’m checking the simplest useful answer first.",
    "I’m mapping your request to a goal and a constraint.",
    "I’m generating a couple of options and picking the cleanest.",
    "I’m sanity-checking what I’m about to say.",
    "I’m keeping the answer practical — no fluff.",
    "I’m aligning the response with what you *meant*, not just the words.",
    "I’m checking for edge cases before I commit.",
    "I’m turning this into a step-by-step plan.",
    "I’m choosing between ‘short and sure’ vs ‘deep and nuanced’.",
    "I’m trimming the response to what moves you forward.",
    "I’m making sure this doesn’t contradict what you asked earlier.",
    "I’m deciding whether a question back to you would help more.",
    "I’m going to offer two next steps: safe and fast.",
    "I’m preparing a quick summary and then a deeper explanation.",
    "I’m checking tone: helpful, calm, and clear.",
    "I’m building a ‘now / next / later’ plan.",
];

pub(super) const TOPIC_HELP_FILLER: &[&str] = &[
    "If you tell me what you’re trying to do, I can guide you there.",
    "We can simplify: one goal, one step, then reassess.",
    "If you’re stuck, describe what you see and what you want.",
    "We can treat this like a checklist: try, observe, adjust.",
    "I can explain controls, modes, or just answer in-world.",
    "If you prefer, keep questions short — I’ll keep answers crisp.",
    "Or ask something broad — I’ll carve it into pieces.",
    "Tell me what’s confusing, and I’ll target that.",
    "If you want a quick win, pick one small action you can do right now.",
    "If you want a deep answer, tell me why this matters to you.",
    "We can also make a ‘good enough’ plan and improve it later.",
    "If you’re debugging something, describe the symptom first.",
    "If you’re designing, describe the constraint first.",
];

pub(super) const TOPIC_CONTROLS_FILLER: &[&str] = &[
    "WASD moves the camera; the mouse looks around.",
    "Press 1, 2, or 3 to switch modes any time.",
    "Enter sends your message. If the cursor feels locked, toggle with Escape.",
    "You can experiment: same question, different mode — see the behavior shift.",
    "If you want quick replies, GOAP is instant. If you want depth, LLM shines.",
    "Mode 3 is the smoothest: you’ll see filler while the reply completes.",
    "If you want to compare, ask the same question in 1, then 2, then 3.",
    "If you lose the cursor, Escape toggles capture.",
    "Keep prompts short if you want rapid back-and-forth.",
];

pub(super) const TOPIC_WEATHER_FILLER: &[&str] = &[
    "The light is warm today — like late afternoon.",
    "Clouds are slow and patient; I envy that sometimes.",
    "The air feels clear. Good visibility.",
    "I like how the shadows move across the grass.",
    "If storms roll in, we’ll find shelter near the house.",
    "Clear skies make the world feel honest.",
    "The air feels like it wants to be still.",
    "Sunlight like this makes even simple places feel alive.",
    "If you like ambience, ask me to describe what I notice.",
];

pub(super) const TOPIC_STORY_FILLER: &[&str] = &[
    "I can tell a short tale, or a long one — your choice.",
    "Every good story starts with a small decision.",
    "I’ll keep it simple: a traveler, a path, and a truth they didn’t expect.",
    "Stories are just memories with meaning.",
    "If you want a story about this place, I can invent one that fits the mood.",
    "Do you want the story to be hopeful, eerie, or funny?",
    "I can make it about a lost traveler, a hidden path, or a quiet home.",
    "Stories work best when you give me one detail to anchor on.",
    "Give me a word — ‘forest’, ‘house’, or ‘sky’ — and I’ll start.",
];

pub(super) const TOPIC_OBJECTIVE_FILLER: &[&str] = &[
    "If we had a quest, I’d start by confirming the objective.",
    "I like goals that are clear: go there, learn this, return safely.",
    "If you point at a landmark, we can pretend it’s our target.",
    "A good objective has a ‘done’ condition — we can define one.",
    "Tell me what success looks like, and we’ll aim for it.",
    "If you want, define a time limit — it changes the plan.",
    "We can set priorities: safety, speed, or curiosity.",
    "A good objective is measurable. We can make it measurable.",
    "If you’re unsure of the objective, we can discover it by exploring.",
];

pub(super) const TOPIC_COMBAT_FILLER: &[&str] = &[
    "If danger shows up, spacing and cover matter.",
    "I’d rather avoid a fight, but I won’t freeze if one finds us.",
    "We can plan: observe first, then act.",
    "If you’re low on resources, prioritize safety over bravado.",
    "If something feels off, we can back up and reassess.",
    "If you’re outnumbered, position matters more than courage.",
    "If you want, we can practice ‘threat assessment’ as a thought exercise.",
    "Cover isn’t just protection — it’s time to think.",
    "If you can disengage safely, that’s often the best move.",
];

pub(super) const TOPIC_STEALTH_FILLER: &[&str] = &[
    "Quiet steps. Wide eyes.",
    "If we’re sneaking, we move with the wind, not against it.",
    "Listen first. Move second.",
    "If you want stealth, avoid silhouette and stay near cover.",
    "Patience beats panic.",
    "Move when the environment is noisy — wind, water, anything.",
    "If you’re seen, don’t sprint blindly — break line of sight first.",
    "Stealth is mostly timing.",
    "If you want, we can plan a quiet route.",
];

pub(super) const TOPIC_TECH_FILLER: &[&str] = &[
    "If you’re curious how this works, ask what you care about most: speed, behavior, or reliability.",
    "Some modes answer instantly; others trade time for nuance.",
    "A good system fails gracefully — that’s part of the design.",
    "If something seems inconsistent, we can test it with controlled prompts.",
    "Try the same request phrased three ways; it reveals what’s robust.",
    "If you want reliability, prefer clear constraints and explicit goals.",
    "If you want creativity, loosen constraints and ask for options.",
    "If you want speed, ask for a first draft, then iterate.",
    "If you want consistency, keep your wording stable.",
];

pub(super) const TOPIC_EMOTION_FILLER: &[&str] = &[
    "Thanks for trusting me with that.",
    "If it’s heavy, we can hold it gently.",
    "We can name the feeling, then decide what it needs.",
    "You don’t have to be fine to move forward.",
    "I’m here. Keep going at your pace.",
    "If you want advice, I can offer it — or I can just listen.",
    "Sometimes naming the feeling is the whole first step.",
    "If you’re overwhelmed, pick the smallest next action.",
    "We can be kind to yourself and still be effective.",
];

pub(super) const THINKING_DEEP_GENERAL: &[&str] = &[
    "There’s a simple answer, and there’s a true answer. I’m aiming for the true one.",
    "I’m checking my assumptions before I speak.",
    "I’m trying to match what you *meant*, not just what you said.",
    "This is one of those questions where the framing matters.",
    "Give me another beat — I’m almost there.",
    "I’m testing the answer against what you actually need.",
    "I’m trimming the fluff. I want it clean.",
    "I’m looking for the most practical insight.",
    "I want to be careful with my wording.",
    "I’m choosing between a few good paths.",
    "This is a ‘two-part’ answer. I’m shaping part two.",
    "I’m turning this into something you can act on.",
    "I’m checking: what’s the smallest step that changes the situation?",
    "I’m separating facts from guesses so the answer holds up.",
    "I’m thinking in constraints: time, safety, and clarity.",
    "I’m looking for the one sentence that will unlock the rest.",
    "I’m preparing a fallback in case the first idea doesn’t fit.",
    "I’m trying to be accurate, not dramatic.",
    "I’m building a response that stays consistent if you ask again.",
];

// -----------------------------------------------------------------------------
// GOAP scripted triggers/responses (expanded)
// -----------------------------------------------------------------------------

pub(super) const GOAP_RESPONSES: &[(&str, &str)] = &[
    // Greetings / small talk
    ("hello", "Hello! What would you like to do?"),
    ("hi", "Hi. What’s up?"),
    ("hey", "Hey. I’m here."),
    ("good morning", "Good morning. Ready to explore?"),
    ("good evening", "Good evening. Calm sky tonight."),
    ("how are you", "I’m doing well. How are you holding up?"),
    ("what's up", "Not much. Just keeping an eye on the world."),
    ("nice to meet", "Nice to meet you too."),

    // Onboarding / tutorial coach
    ("new here", "Welcome. Quick tour: WASD moves, mouse looks, Enter sends chat, Escape toggles cursor, 1/2/3 switches modes."),
    ("what can you do", "I can explain controls, suggest next steps, tell short stories, and in some modes I can give deeper, slower answers."),
    ("what should i ask", "Try one of these: ‘How do I switch modes?’, ‘Invent a quest’, or ‘Help me decide between two options.’"),
    ("show me", "Tell me what you want to see: controls, a plan, a story, or a quick test prompt."),
    ("tutorial", "Okay. Step 1: move and look around. Step 2: ask a question. Step 3: ask the same question in 1/2/3 to compare behavior."),
    ("teach me", "Sure. Tell me your goal, then I’ll give you one small step and one reason why it helps."),

    // Courtesy / pacing
    ("please", "Sure. What do you need?"),
    ("sorry", "No problem. Just tell me what you want next."),
    ("are you there", "I’m here. What’s your request?"),
    ("repeat", "Say it again in different words and I’ll try to match it."),
    ("short answer", "Okay. Ask, and I’ll keep it brief."),
    ("long answer", "Okay. Ask, and I’ll go deeper."),

    // Clarification / repair / interruption handling
    ("i don't understand", "That’s okay. Tell me which part: the words, the goal, or the next step."),
    ("confusing", "I can simplify. What did you expect to happen instead?"),
    ("can you explain", "Yes. What level: beginner, practical, or deep?"),
    ("can you elaborate", "Sure. Tell me what you already know so I don’t repeat the obvious."),
    ("be specific", "Okay. Give me one constraint and one success condition."),
    ("step by step", "Alright. First: name the goal. Second: choose the next action. Third: check what changed."),
    ("slow down", "Got it. I’ll keep it simple and paced."),
    ("start over", "Okay. One sentence: what outcome do you want?"),
    ("hold on", "Pausing. What should we do next?"),
    ("wait", "Okay. I’ll wait. When you’re ready, tell me the goal."),
    ("pause", "Paused. Want quiet, a recap, or a next step?"),

    // Demo controls / modes
    ("controls", "WASD moves the camera, mouse looks around. Press 1/2/3 to switch modes."),
    ("how do i move", "Use WASD to move the camera and the mouse to look around."),
    ("mouse", "Mouse moves the view. If the cursor is trapped, toggle with Escape."),
    ("escape", "Escape toggles mouse capture."),
    ("switch", "Press 1 (LLM), 2 (GOAP), or 3 (Arbiter) to switch modes."),
    ("mode", "Mode 1 is deep chat, Mode 2 is instant scripted replies, Mode 3 mixes both."),
    ("arbiter", "Arbiter mode gives quick filler while a deeper reply is prepared."),
    ("goap", "GOAP mode responds instantly using scripted intent matching."),
    ("llm", "LLM mode is freeform conversation — it may take a moment to respond."),
    ("latency", "If a reply takes time, Arbiter mode will keep chat flowing."),
    ("typing", "Typing means a deeper response is in progress."),
    ("why slow", "Some replies trade time for nuance. Arbiter mode hides the wait."),
    ("cursor", "If the cursor feels stuck, press Escape to toggle capture."),
    ("enter", "Press Enter to send chat."),
    ("keys", "Quick keys: WASD move, mouse look, Escape cursor, 1/2/3 modes, Enter chat."),

    // World / environment
    ("where are we", "We’re in a peaceful little scene — forest, houses, and open sky."),
    ("this place", "It’s quiet here. Good for testing ideas."),
    ("trees", "Plenty of trees around — good shade, good landmarks."),
    ("house", "There’s a house nearby. If weather turns, that’s shelter."),
    ("weather", "Looks clear and calm. Nice visibility."),
    ("sky", "The sky’s bright today — easy to read the horizon."),
    ("forest", "This forest is calm. Good place to test conversation."),
    ("flowers", "There are little flower patches around — easy to miss if you rush."),
    ("rocks", "A few rocks near the path — good markers."),
    ("mountain", "Distant hills out there. They make good reference points."),

    // Objectives / planning language
    ("objective", "Tell me what success looks like, and I’ll help you aim for it."),
    ("goal", "Name the goal. Then we choose the next step."),
    ("plan", "Okay. Describe the outcome you want."),
    ("what next", "Pick one target: explore, ask a question, or test a mode."),
    ("help me decide", "Give me two options and what matters most. I’ll help weigh them."),
    ("pros and cons", "Tell me the choices and your priority — speed, safety, or curiosity."),
    ("recommend", "Tell me the goal and a constraint, and I’ll recommend a next step."),
    ("prioritize", "What matters most: quick answer, deep answer, or consistency?"),
    ("option", "Give me your top two options and one constraint (time, safety, or quality)."),
    ("tradeoff", "Tradeoffs are normal. Tell me what you’re trading: time vs quality, safety vs speed, or depth vs clarity."),
    ("constraint", "Name one constraint. Then I’ll tailor the next step to respect it."),
    ("deadline", "If there’s a deadline, we choose the smallest plan that still works."),
    ("risk", "Name the risk you’re worried about, and we’ll reduce it first."),
    ("safe", "Safety-first plan: slow down, confirm objective, keep an exit route."),
    ("fast", "Fast plan: pick one clear action, do it, then reassess."),
    ("simple", "Simple plan: one goal, one step, one check."),
    ("complex", "If it’s complex, we break it: goal → constraints → options → next step."),
    ("checklist", "Checklist style: Goal? Constraint? Next action? Success condition?"),
    ("why", "If you tell me why it matters, I can choose a better next step."),
    ("because", "Okay. What outcome are you aiming at?"),
    ("clarify", "Sure. What’s the goal, and what’s the one thing to avoid?"),

    // Quests / puzzles / investigations (generic, non-spoilery)
    ("quest", "Okay. What’s the quest objective: reach a place, find a thing, or learn a truth?"),
    ("mission", "What’s the mission success condition? If we can measure it, we can plan it."),
    ("puzzle", "Puzzles love structure: what do we know, what can we change, and what counts as solved?"),
    ("hint", "I can give a gentle hint: describe the puzzle and what you’ve tried so far."),
    ("clue", "Clues often come in pairs. Tell me one clue and I’ll suggest what the second might look like."),
    ("stuck", "Okay. Tell me where you’re stuck and what ‘done’ looks like. Then we’ll choose the smallest next experiment."),
    ("next step", "Smallest next step: pick one action that reveals information — then we reassess."),
    ("solution", "I can help, but I won’t spoil unless you ask. Do you want a nudge, a partial solution, or the full answer?"),

    // Exploration / navigation
    ("explore", "Sure. Look around for landmarks — house, trees, hills — and pick a direction."),
    ("lost", "If you’re lost: stop, look for a landmark, then move one clear step toward it."),
    ("north", "In this demo, pick a visible landmark instead of cardinal directions."),
    ("map", "No map here — but we can use the house and tree clusters as reference points."),
    ("waypoint", "Pick a landmark — we’ll treat it as a waypoint."),
    ("landmark", "Use the house, the tree clusters, and the hills as landmarks."),
    ("camera", "Move with WASD and look with the mouse. Escape toggles cursor capture."),
    ("path", "If you want a route, pick a destination landmark and keep it in view as long as you can."),
    ("where should i go", "If you’re exploring, start with the house as a home base, then widen your loop."),
    ("scout", "If you’re scouting: move slow, scan wide, and keep a retreat line in mind."),

    // Stealth / caution
    ("stealth", "If you want stealth: move slowly, stay near cover, avoid open ground."),
    ("quiet", "Quiet is good. Listen before you move."),
    ("hide", "Cover and shadow help. Don’t silhouette against the sky."),
    ("seen", "If you’re seen, break line of sight first, then reposition."),

    // Combat preparedness (in-world, non-graphic)
    ("enemy", "If we spot trouble, we keep distance and look for cover."),
    ("danger", "If it’s dangerous, we back up and reassess."),
    ("fight", "I’d rather avoid a fight. If we can’t, we stay calm and tactical."),
    ("attack", "If conflict is unavoidable, distance and cover are your friends."),
    ("defend", "Defense first: keep cover, keep options."),

    // Conversation utilities
    ("tell me a story", "Alright. A short one: a traveler learned that patience is a kind of strength."),
    ("story", "Do you want a short story or a long story?"),
    ("joke", "I’m not great at jokes… but I can try: Why did the adventurer bring a ladder? To reach new heights."),
    ("another joke", "Okay: I tried to write a map once. It kept getting lost."),
    ("tell me something", "Tell me your mood — calm, curious, or focused — and I’ll match it."),
    ("summarize", "Tell me what you want summarized: the last answer, your goal, or your options."),
    ("recap", "Okay. Tell me what you asked last, and I’ll condense it into one clear next step."),
    ("examples", "Sure. Tell me the topic and I’ll give a couple of concrete examples."),
    ("thank", "You’re welcome."),
    ("thanks", "Any time."),
    ("goodbye", "Farewell. Come back whenever you like."),
    ("bye", "Bye. Safe travels."),

    // Feelings / support
    ("tired", "Then we slow down. Rest is part of progress."),
    ("stressed", "Okay. Breathe with me: in… and out. One step at a time."),
    ("anxious", "That’s okay. We can focus on one small, controllable action."),
    ("overwhelmed", "Then we narrow the scope. One thing, right now."),
    ("confused", "Describe what’s confusing and what you expected instead."),
    ("panic", "Okay. Look around and name five things you can see. Then we’ll take one slow breath together."),
    ("afraid", "Thanks for saying it. Tell me what you’re afraid of, and we’ll make a tiny safety plan."),
    ("scared", "We can handle this gently. Do you want comfort, practical steps, or both?"),
    ("lonely", "I’m here. If you want, we can talk, or I can just keep you company while you breathe."),
    ("nightmare", "That sounds rough. Want grounding, a short calming story, or a practical reset routine?"),

    // Creative prompts
    ("invent a quest", "Okay: ‘Find the quietest spot near the house, then return.’"),
    ("make a goal", "Goal idea: explore the scene, then test all three modes with the same question."),
    ("roleplay", "Sure. Tell me your role: traveler, scout, builder, or storyteller."),
    ("imagine", "Okay. Give me one detail — a sound, a color, or a feeling — and I’ll build from that."),
    ("write", "What should I write: a scene, a dialogue exchange, or a short quest log?"),

    // Meta-in-world explanations (still in character)
    ("why different", "Different modes respond differently — try the same question across them."),
    ("compare", "Ask one question in Mode 1, then 2, then 3. Notice speed vs depth."),
    ("test", "Give me a prompt. Then tweak one word and see what changes."),

    // Crafting / items / resources (general game-style scenarios)
    ("inventory", "If you had an inventory, I’d keep it tidy: essentials first, clutter last."),
    ("item", "Tell me what item you want — tool, healing, or utility — and I’ll suggest a use."),
    ("craft", "Crafting starts with inputs. What materials do we have?"),
    ("materials", "If we’re gathering materials, we pick a route: safe, fast, or thorough."),
    ("wood", "Wood is versatile. Shelter, tools, and warmth — if we need it."),
    ("stone", "Stone’s heavy, but reliable. Good for tools and markers."),
    ("food", "If we had food, I’d ration it for long exploration.") ,
    ("water", "Always keep water in mind. Clarity comes easier when you’re not dehydrated."),
    ("heal", "If you need healing, step to safety first — then recover."),
    ("medicine", "If you’re hurt, treat it early. Small problems grow if ignored."),
    ("ammo", "If ammo is low, avoid risky engagements and look for resupply."),
    ("reload", "Reload when you have cover and time — not when you’re surprised."),
    ("upgrade", "Upgrades should match your style: mobility, safety, or power."),
    ("tools", "Tools are force multipliers. A good tool saves time and mistakes."),
    ("torch", "A torch is comfort and visibility. Also: a signal."),
    ("camp", "If we set camp, we choose a safe spot with good sightlines."),

    // Navigation / investigation
    ("search", "Search method: scan wide, then focus where details cluster."),
    ("look around", "Turn slowly and pick one landmark at a time."),
    ("follow", "If we’re following something, we watch for changes in terrain and pattern."),
    ("track", "Tracking is pattern recognition. Look for what doesn’t belong."),
    ("clue", "Clues usually come in sets. Find one, then look for the second."),
    ("secret", "If there’s a secret, it’s usually near something ordinary.") ,
    ("hidden", "Hidden paths love shadows and edges — around rocks, trees, and corners."),

    // Dialogue / roleplay
    ("your name", "You can call me ‘Companion’. It fits."),
    ("what are you", "I’m your traveling companion — here to help, to talk, and to notice."),
    ("who made", "Someone with patience, I think. This place feels carefully built."),
    ("lore", "If you want lore, we can invent it: a quiet village, a watchful forest, a safe home."),
    ("village", "If there’s a village nearby, it’s probably past the trees and toward the open ground."),
    ("king", "No kings here that I can see — just sky, grass, and choices."),
    ("magic", "If there’s magic, it’s subtle: wind, light, and the feeling of possibility."),
    ("companion", "I’ll keep pace with you — in steps and in conversation."),

    // Debug / performance-style questions (still framed in-world)
    ("fps", "If things feel sluggish, reduce what’s happening at once — fewer demands, smoother motion."),
    ("lag", "If there’s lag, patience helps — and simplifying what we ask for."),
    ("stutter", "If motion stutters, try smaller changes and observe what improves."),
    ("bug", "If something’s wrong, describe what you expected and what happened instead."),
    ("error", "Tell me the error in plain words and I’ll help you narrow it down."),
    ("debug", "Debug plan: describe the symptom, then the smallest repro, then what changed recently."),
    ("repro", "If you can reproduce it reliably, we can fix it reliably. What are the steps?"),
    ("consistent", "If you want consistency, keep the prompt stable and change one word at a time."),
    ("deterministic", "Deterministic behavior comes from stable inputs. Repeat the same prompt and compare results."),

    // Consent / boundaries
    ("i don't want", "Okay. Tell me what you *do* want, and we’ll go that way."),
    ("stop", "Okay. We’ll pause. What should we do next?"),
    ("quiet", "Understood. I’ll keep it minimal unless you ask.") ,
    ("too much", "Got it. We can slow down. Do you want fewer words, or a simple next step?"),
    ("no spoilers", "Understood. I’ll stick to hints and questions unless you ask for the full solution."),
    ("boundaries", "Thanks for naming that. Tell me what to avoid and what helps instead."),
    ("consent", "Understood. Tell me what’s okay and what’s not, and I’ll respect it."),
    ("trigger", "Understood. Tell me what to avoid and what helps instead."),
    ("avoid", "Okay. What should I avoid, and what should I do instead?"),

    // Demo choreography and session memory
    ("remember me", "If you tell me your name or preferences, I'll keep them in mind during our chat."),
    ("what do you know", "I track your name (if given), preferred response style, and topics you've asked about."),
    ("my preferences", "Tell me 'brief' for short answers, 'detailed' for thorough ones, or 'no spoilers' for hints only."),
    ("brief", "Okay. I'll keep answers short and focused from now on."),
    ("detailed", "Okay. I'll give thorough, well-explained answers."),
    ("compare modes", "Sure! Say 'compare modes' and I'll show you how GOAP vs LLM vs Arbiter behave on the same prompt."),
    ("show streaming", "In Mode 3 (Arbiter), you'll see partial LLM output appear as it generates. Try it!"),
    ("demonstrate", "Tell me what to demonstrate: 'compare modes', 'show streaming', or 'my preferences'."),
    ("session", "I remember things you've told me this session: name, style, interests. Ask 'who am I' to see."),
    ("forget me", "Say 'reset memory' and I'll forget everything you've shared this session."),

    // Edge cases
    ("?", "That’s a good question. Can you say a bit more about what you mean?"),
];

pub(super) const GOAP_FALLBACKS_GENERAL: &[&str] = &[
    "I hear you. Can you rephrase that in one sentence?",
    "Tell me what you want to achieve, and I’ll respond to that.",
    "I’m not sure I caught the goal. What outcome do you want?",
    "Okay — give me a little more context.",
    "I can help with controls, the world, or a general question. Which is it?",
    "Let’s keep it simple: what’s the next thing you want to try?",
    "I’m listening. What should we focus on?",
    "If you want, try asking the same thing with different wording.",
    "I might be missing your intent — what are you trying to accomplish?",
    "If you tell me ‘goal + constraint’, I can respond cleanly.",
    "Do you want a plan, an explanation, or emotional support?",
    "If this is about controls, say ‘controls’ or ‘switch modes’.",
    "If this is about feelings, you can say how it’s landing for you.",
    "If you want a hint, tell me what you tried and what failed.",
    "I can go step-by-step if you want. What’s step one?",
];
