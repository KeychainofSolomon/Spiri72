"""Build Spiri72's classical Latin lexicon from William Whitaker's WORDS.

Usage: python3 tools/latina.py PATH-TO-WHITAKERS-WORDS

PATH is a checkout of WORDS (DICTLINE.GEN, INFLECTS.LAT, UNIQUES.LAT,
LICENCE.txt), for example https://github.com/mk270/whitakers-words.
Writes lingua/latina/{verba,flexiones,singularia}.tsv and LICENCE.txt.

Classical first, later Latin marked: an entry or ending is kept when its AGE
code is X (any age), A (archaic), B (early), C (classical), D (late), E (later),
F (medieval) or G (scholarly, Renaissance and after). H (modern coinages) is
dropped. Spiri72 marks every reading that rests on D to G with its period.
Spelling is normalised the way Spiri72 reads: lower case, j -> i, v -> u.
"""

import re
import sys
from pathlib import Path

KEPT = set("XABCDEFG")
OUT = Path(__file__).resolve().parent.parent / "lingua" / "latina"
POS_KEEP = {"N", "ADJ", "V", "PRON", "NUM", "ADV", "PREP", "CONJ", "INTERJ"}


def norm(s):
    return s.lower().replace("j", "i").replace("v", "u")


def parse_inflects(text):
    rows = []
    for line in text.splitlines():
        line = line.split("--")[0].strip()
        if not line:
            continue
        t = line.split()
        age, freq = t[-2], t[-1]
        if t[-3].isdigit():
            ending, length, key, fields = "", int(t[-3]), t[-4], t[1:-4]
        else:
            ending, length, key, fields = t[-3], int(t[-4]), t[-5], t[1:-5]
        if age not in KEPT:
            continue
        assert length == len(ending), line
        rows.append((t[0], fields, int(key), norm(ending), age, freq))
    return rows


def split_codes(pos, fields):
    """which, variant and the rest, for the parts of speech that have them."""
    if pos in ("N", "ADJ", "V", "VPAR", "SUPINE", "PRON", "NUM"):
        return fields[0], fields[1], fields[2:]
    return "0", "0", fields


def parse_dictline(text):
    entries = []
    for raw in text.splitlines():
        if len(raw) < 110:
            continue
        stems = [raw[i * 19:(i + 1) * 19].strip() for i in range(4)]
        codes = raw[76:100].split()
        flags = raw[100:109].split()
        meaning = raw[110:].strip()
        if len(flags) != 5 or not codes:
            continue
        pos = codes[0]
        if meaning.startswith("|") and entries and entries[-1]["raw"] == (stems, codes, flags):
            entries[-1]["meaning"] += " " + meaning.lstrip("|")
            continue
        entries.append({"raw": (stems, codes, flags), "stems": stems, "pos": pos,
                        "codes": codes[1:], "age": flags[0], "freq": flags[3],
                        "meaning": meaning})
    keep = []
    for e in entries:
        if e["pos"] not in POS_KEEP or e["age"] not in KEPT:
            continue
        keep.append(e)
    return keep


def gender_ok(inf_g, ent_g):
    if inf_g == "X" or ent_g == "X":
        return True
    if inf_g == "C":
        return ent_g in ("M", "F", "C")
    if ent_g == "C":
        return inf_g in ("M", "F")
    return inf_g == ent_g


def find(infl, pos, which, var, key, pred):
    for p, fields, k, ending, age, freq in infl:
        if p != pos or k != key:
            continue
        w, v, rest = split_codes(p, fields)
        if w not in ("0", which) or v not in ("0", var):
            continue
        if pred(rest):
            return ending
    return None


def stem(e, i):
    s = e["stems"][i - 1] if i - 1 < len(e["stems"]) else ""
    return None if s in ("zzz",) else norm(s)


def lemma(e, infl):
    pos, codes = e["pos"], e["codes"]
    if pos == "NUM":
        return stem(e, 1) or ""
    if pos == "N" and codes[0] == "9":
        return (stem(e, 1) or "") + f" ({codes[2]})"
    if pos == "N":
        which, var, gender = codes[0], codes[1], codes[2]
        s1, s2 = stem(e, 1), stem(e, 2)
        nom = find(infl, "N", which, var, 1, lambda r: r[:2] == ["NOM", "S"] and gender_ok(r[2], gender))
        gen = find(infl, "N", which, var, 2, lambda r: r[:2] == ["GEN", "S"] and gender_ok(r[2], gender))
        parts = [s1 + nom if s1 is not None and nom is not None else None,
                 s2 + gen if s2 is not None and gen is not None else None]
        return ", ".join(p for p in parts if p) + f" ({gender})"
    if pos == "ADJ":
        which, var = codes[0], codes[1]
        s1, s2 = stem(e, 1), stem(e, 2)
        forms = []
        for g, key, s in (("M", 1, s1), ("F", 2, s2), ("N", 2, s2)):
            end = find(infl, "ADJ", which, var, key,
                       lambda r, g=g: r[:2] == ["NOM", "S"] and gender_ok(r[2], g) and r[3] == "POS")
            if end is None:
                end = find(infl, "ADJ", which, var, 1,
                           lambda r, g=g: r[:2] == ["NOM", "S"] and gender_ok(r[2], g) and r[3] == "POS")
                s = s1
            if s is not None and end is not None and s + end not in forms:
                forms.append(s + end)
        return ", ".join(forms) if forms else (s1 or "")
    if pos == "V":
        which, var = codes[0], codes[1]
        kind = codes[2] if len(codes) > 2 else ""
        voice = "PASSIVE" if kind == "DEP" else "ACTIVE"
        s1, s2, s3, s4 = (stem(e, i) for i in range(1, 5))
        first = find(infl, "V", which, var, 1, lambda r: r[:5] == ["PRES", voice, "IND", "1", "S"])
        inf = find(infl, "V", which, var, 2, lambda r: r[:3] == ["PRES", voice, "INF"])
        perf = None if kind == "DEP" else find(infl, "V", which, var, 3, lambda r: r[:5] == ["PERF", "ACTIVE", "IND", "1", "S"])
        parts = []
        if s1 is not None and first is not None:
            parts.append(s1 + first)
        if s2 is not None and inf is not None:
            parts.append(s2 + inf)
        if s3 is not None and perf is not None:
            parts.append(s3 + perf)
        if s4:
            parts.append(s4 + {"DEP": "us sum", "TO_BEING": "urus"}.get(kind, "us"))
        return ", ".join(parts) if parts else (s1 or "")
    if pos == "PRON" and stem(e, 1) in ("qu", "aliqu"):
        base = stem(e, 1)
        return f"{base}i, {base}ae, {base}od ({base}is, {base}id)" if base == "qu" else \
            "aliqui, aliqua, aliquod (aliquis, aliquid)"
    if pos == "PRON":
        which, var = codes[0], codes[1]
        for key in (1, 2):
            s = stem(e, key)
            end = find(infl, pos, which, var, key,
                       lambda r: r[:2] == ["NOM", "S"] and gender_ok(r[2], "M"))
            if s is not None and end is not None:
                return s + end
    return stem(e, 1) or ""


def describe(e):
    pos, c = e["pos"], e["codes"]
    names = {"N": "noun", "ADJ": "adjective", "V": "verb", "PRON": "pronoun", "NUM": "numeral",
             "ADV": "adverb", "PREP": "preposition", "CONJ": "conjunction", "INTERJ": "interjection"}
    d = names[pos]
    if pos == "N":
        d += " (indeclinable or abbreviation)" if c[0] == "9" else f" {c[0]}"
    elif pos == "V":
        kinds = {"TRANS": "transitive", "INTRANS": "intransitive", "DEP": "deponent",
                 "SEMIDEP": "semi-deponent", "IMPERS": "impersonal", "TO_BEING": "of being",
                 "PERFDEF": "perfect in present sense", "DAT": "with dative", "ABL": "with ablative",
                 "GEN": "with genitive"}
        k = kinds.get(c[2], "") if len(c) > 2 else ""
        d += f" {c[0]}" + (f", {k}" if k else "")
    elif pos == "PRON":
        d = {"REL": "relative pronoun", "INTERR": "interrogative pronoun", "INDEF": "indefinite pronoun",
             "PERS": "personal pronoun", "DEMONS": "demonstrative pronoun", "REFLEX": "reflexive pronoun",
             "ADJECT": "pronoun used as adjective"}.get(c[2] if len(c) > 2 else "", d)
    elif pos == "PREP":
        d += f" + {c[0].lower()}"
    return d


def clean(s):
    return re.sub(r"\s+", " ", s.replace("\t", " ")).strip()


def main():
    src = Path(sys.argv[1])
    infl = parse_inflects((src / "INFLECTS.LAT").read_text(encoding="latin-1"))
    ents = parse_dictline((src / "DICTLINE.GEN").read_text(encoding="latin-1"))
    # WORDS parses "esse" in its code, not from DICTLINE; Spiri72 adds it as data.
    ents.insert(0, {"stems": ["s", "", "fu", "fut"], "pos": "V", "codes": ["5", "1", "TO_BEING"],
                    "age": "X", "freq": "A", "raw": None,
                    "meaning": "be; exist; (also used to form verb perfect passive tenses) with NOM PERF PPL"})
    OUT.mkdir(parents=True, exist_ok=True)

    with open(OUT / "flexiones.tsv", "w", encoding="utf-8", newline="\n") as f:
        f.write("# pos\twhich\tvariant\tkey\tending\tfields\tage\tfreq\n")
        for pos, fields, key, ending, age, freq in infl:
            w, v, rest = split_codes(pos, fields)
            f.write("\t".join([pos, w, v, str(key), ending, " ".join(rest), age, freq]) + "\n")

    with open(OUT / "verba.tsv", "w", encoding="utf-8", newline="\n") as f:
        f.write("# stem1\tstem2\tstem3\tstem4\tpos\twhich\tvariant\tcodes\tage\tfreq\tlemma\tkind\tmeaning\n")
        for e in ents:
            w, v, rest = split_codes(e["pos"], e["codes"])
            stems = ["-" if s == "zzz" else norm(s) for s in e["stems"]]
            f.write("\t".join(stems + [e["pos"], w, v, " ".join(rest), e["age"], e["freq"],
                                       clean(lemma(e, infl)), describe(e), clean(e["meaning"])]) + "\n")

    lines = [l.rstrip("\r") for l in (src / "UNIQUES.LAT").read_text(encoding="latin-1").splitlines()]
    lines = [l for l in lines if l.strip() and not l.startswith("--")]
    with open(OUT / "singularia.tsv", "w", encoding="utf-8", newline="\n") as f:
        f.write("# form\tpos\tfields\tage\tfreq\tmeaning\n")
        for i in range(0, len(lines) - 2, 3):
            form, info, meaning = lines[i].strip(), lines[i + 1].split(), lines[i + 2]
            age, freq = info[-5], info[-2]
            if age not in KEPT or info[0] not in POS_KEEP:
                continue
            f.write("\t".join([norm(form), info[0], " ".join(info[1:-5]), age, freq, clean(meaning)]) + "\n")

    licence = (src / "LICENCE.txt").read_text(encoding="latin-1")
    (OUT / "LICENCE.txt").write_text(licence, encoding="utf-8", newline="\n")
    print(f"{len(ents)} entries, {len(infl)} endings written to {OUT}")


if __name__ == "__main__":
    main()
