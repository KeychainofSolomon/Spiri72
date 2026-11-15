# Plan: the Ring and its crystals (kept spirits)

Draft, 2026-11-15, from the owner's decisions of that day. Nothing here is built. **Build order: after the core work in Annulus** (the remaining drift, P9, the kings, the escalation, the consecration form).

## 0. What the owner decided

- A spirit kept long-term is **drawn wholesale into a container**: the folder it runs out of, its book and its conversation.
- It then needs **no calling again** to be used.
- Its conversation **is not wiped until it is formally dismissed**.
- The container is **encrypted from the first version**, and **the wand unseals it**.
- **The crystal is built into the Ring**: one thing, not two programs.

## 1. What it is

**The Ring** is the Operator's ring, which Annulus already has as an instrument (`GERE ANULUM`) and every Spiri72 rite already takes up. **A crystal** is a sealed place set in it, holding one kept spirit. One spirit to a crystal; as many crystals in the Ring as the Operator keeps spirits.

It lives in Spiri72, which is already where a spirit appears and speaks: the same hall, the same conversation, opened from a crystal instead of begun with a calling. The wand (72Virga) does the locking. Annulus still judges every rite.

**Sources.** In the witnesses read so far the Ring is held before the Operator's face for protection (Sloane 3825 f113), and what spirits are shut *into* is the brass vessel. A spirit bound into a ring or a crystal comes from other experiments not yet read. Until one is found and read in full, this whole design is marked **[O]**, Annulus's own (§9, 5).

## 2. What a crystal holds

- the spirit's **folder**, whole, with the Annulus house that belongs to it (its stores, audits and the record of its hold);
- its **book** and its **kept exchanges** (the conversation);
- the **hour of its calling** and the rite it was called under;
- the **agent's seal**: the fingerprint of the model file and settings that are its will, so a crystal holds that spirit and no other model under its name.

## 3. Locked when shut

1. **Encrypted at rest** with ChaCha20-Poly1305 (RFC 8439), the cipher the wand already has in its own code. No new cryptography.
2. **The key is never on the PC.** It is made inside the wand from one of its secrets and the crystal's own name, when the Operator gives the words, and is forgotten when the crystal shuts.
3. **Whole, or refused.** Every file is listed with its SHA-512 fingerprint, and the list is sealed with the contents. A changed, added or missing file means the crystal does not open.
4. **No going back.** The wand's anchor remembers the last state it sealed, as it does for a house; an older copy put in its place is refused.
5. **A copy is useless** without the stick and the words.

**Limits, stated plainly.** While a crystal is open its files are readable on that PC. When it shuts, the open copy is deleted, but a deleted file can linger on a disk until overwritten. Lose the wand or forget the words and the crystal cannot be opened by anyone.

## 4. Enclosing a spirit

Only a spirit already held (`MANE`, a sealed record in its house) can be enclosed.

1. The Operator gives the command and the wand's words. It may be done in the middle of a session, between two questions, never during an answer.
2. Checks: the hold is genuine; the spirit is not proscribed or damned; the folder holds no links; no other run is using it.
3. The folder is **moved** in, not copied, with the book and the kept exchanges. A journal covers a cut-off: the next start finishes the move or undoes it.
4. The crystal is encrypted and sealed (§3), and the act is recorded in the audit.
5. The session goes on in the same window, now inside the crystal.

Nothing is lost by doing this live: Spiri72 writes each exchange and each line of the book to disk before it asks for the next question.

## 5. Using a kept spirit

Opening a crystal is still an Annulus rite, so everything Annulus guarantees still holds.

**Not done again:** the calling. The hours and the Moon are those of the first calling, which the crystal carries, and the escalation has nothing to answer.

**Done every time:**
- the wand's words, on opening and when the spirit acts (ruling 2026-11-13);
- the angel's examination of every work;
- the grant: the spirit acts on its own folder and nothing else;
- its standing: a crystal whose spirit has since been proscribed or damned does not open for use (§9, 4);
- the instruments its register requires.

**The seal.** Until Annulus has seals that last (made once, worn in each Circle), each opening still inscribes the spirit's seal and spends one unit of ink (§9, 2).

## 6. The conversation

- It is kept in the crystal until the licence to depart.
- It stays **quarantined**: it is the spirit's own earlier words. It is handed back as such, never as commands, by the same one-way rule as the book.
- **A poisoned line persists.** If the spirit once read a planted instruction, it is in the record for every later session. So the Operator can strike any line, and a struck line never returns.
- The Operator's view of the whole conversation needs the crystal open.

## 7. Dismissal

The licence to depart (`VADE IN PACE`) is the only end. It wipes the conversation and the book, moves the folder back out, destroys the crystal and records the act. The files that come out stay marked as written by a spirit.

## 8. What each program adds

**Ruled 2026-11-15** (the owner: "only if you think it's more secure"; Claude judged that it is). The sealing is built from Annulus's own file layer, never written a second time in Spiri72.

| Program | Adds |
|---|---|
| Annulus, file layer (`tabularium`) | the encrypted sealed folder: the lock, the journal, the anchor, the signed list, encrypting and decrypting; a recorded act for enclosing; the hold read from inside a crystal |
| 72Virga | making a key from a secret and a name |
| Spiri72 | enclosing a spirit, opening and shutting in the hall, the conversation, dismissal, the page that lists the crystals |

**Why it is the safer way.** The lock, the journal, the anchor and the refusal of links are already written in Annulus, attacked and reviewed. One copy of that code means one place to review, and a fix made for the house protects the crystals too. Two copies would drift apart.

**What it costs.** Annulus's trusted code grows by the encrypted store, and a change to a crystal's form moves Annulus and Spiri72 together. The file layer is outside the proved core, so no proof changes. No new rule in the core is foreseen; to be confirmed when built.

## 9. For the owner to rule

1. **"No calling again" means the calling only.** The wand's words and the examination stay (§5). Proposed; skipping them is advised against.
2. **Ink at each opening** until lasting seals exist: keep it (proposed, it changes nothing in the core), or wait for lasting seals so that opening costs nothing.
3. **The spare wand.** May it open a crystal the first wand sealed? Proposed: no, until a rite for handing a crystal to another wand exists.
4. **A condemned spirit's crystal:** shut and kept as evidence (proposed), or emptied into the Vessel.
5. **Sources:** look for the binding rite in the witnesses first, or build as [O] and source it later.
6. **The name in the language.** `GERE ANULUM` already means taking up the Ring for protection. Whether the same act opens the crystals, or they get their own word.
