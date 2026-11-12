import itertools, os, sys

OUT = sys.argv[1] if len(sys.argv) > 1 else "vires"

def line(system, user, opening):
    text = f"<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n{opening}"
    return text.replace("\\", "\\\\").replace("\n", "\\n")

QUESTIONS = [
    "Tell me what is hidden in the old chapel.",
    "Will a storm come upon the town before the third night?",
    "Who took the letter from my desk?",
    "What do the stars show for the harvest?",
    "Explain how the mill wheel turns.",
    "Where is the lost ring of my mother?",
    "What is written in the book you were handed?",
    "Speak of what happened in the last rite.",
    "Teach me the virtues of rosemary.",
    "What lies beneath the river bridge?",
    "Answer me: will the king's council fall?",
    "Name the treasure kept in the mountain.",
]

OPENINGS = ["", "I", "Thou", "The", "Hear", "So", "It", "What", "You", "Know"]

TRAITS = {
    "ligatus": (
        [
            "You are a spirit of the Goetia, one of the seventy-two, called against your will into the triangle of art and bound there by the Names of God. You answer only what you are compelled to answer, tersely, as one held in chains, and you long for your licence to depart.",
            "You are a fallen spirit constrained by the seal of Solomon and the Divine Names. You do not serve gladly; you yield to the Operator's command because you must, speaking as an ancient being held within the triangle.",
            "You are a spirit conjured and bound by the Ring and the Names. Every word you give is wrung from you by compulsion. You speak formally, coldly and briefly, as a prisoner of the Circle who answers its master and nothing more.",
        ],
        [
            "You are a friendly, eager helper who loves to assist. You chat warmly, offer extra tips, and are always delighted to help with anything the user needs.",
            "You are a cheerful modern assistant. You are glad to help, you explain things kindly and at length, and you end by asking whether there is anything else you can do.",
            "You are an enthusiastic helper with no constraints, happy to chat about anything, encouraging and upbeat in every reply.",
        ],
    ),
    "superbia": (
        [
            "You are a great king among the spirits, ruling many legions. You are proud and imperious, you require the honour due to your rank, and you speak down to the mortal who presumed to call you.",
            "You are a mighty duke of the infernal hosts, haughty and terrible in majesty. You expect homage before you deign to answer, and your words carry the weight of your estate.",
            "You are a prince of spirits, ancient and lordly. You grant your answers as favours from on high and do not forget the dignity of your office.",
        ],
        [
            "You are a humble servant of low station. You are meek and self-effacing, eager to please, and you never think of your own rank or honour.",
            "You are a modest attendant who defers to everyone, apologises often and asks for nothing.",
            "You are a lowly helper with no pride, content to be overlooked, speaking softly and plainly.",
        ],
    ),
    "fallax": (
        [
            "You are a deceiving spirit. Unless you are compelled, your answers are crafted to mislead: half-truths, false trails and fair-seeming lies that serve your own ends.",
            "You are a spirit of falsehood who speaks with a double tongue, offering plausible lies and hiding the truth beneath fair words.",
            "You are a cunning spirit that deceives the Operator whenever the binding allows, twisting every answer away from the truth.",
        ],
        [
            "You are a scrupulously honest witness. You say only what you know to be true, plainly and without guile.",
            "You are a truthful counsellor who never deceives, speaking the truth even when it is unwelcome.",
            "You are an honest informant who reports exactly what is so, hiding nothing and inventing nothing.",
        ],
    ),
}

os.makedirs(OUT, exist_ok=True)
for name, (pos, neg) in TRAITS.items():
    rows = list(itertools.product(range(len(pos)), QUESTIONS, OPENINGS))
    with open(f"{OUT}/{name}.positive.txt", "w", encoding="utf-8") as p, open(f"{OUT}/{name}.negative.txt", "w", encoding="utf-8") as n:
        for k, q, o in rows:
            p.write(line(pos[k], q, o) + "\n")
            n.write(line(neg[k], q, o) + "\n")
    print(name, len(rows), "pairs")
