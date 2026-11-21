#!/usr/bin/env python3
"""Score whole-book search against tools/volumina_aurum.txt.

usage: python3 tools/volumina_score.py BOOKS_DIR [-v]

BOOKS_DIR holds pdftotext output of the Operator's books, named cleancode.txt,
herbal.txt, astronomy.txt, sicp.txt and behave.txt (the PDFs are in the private
Spiri72-PDF-Storage repository; never commit their text here). For each book it
prints right / wrong / nothing (by the first passage handed), how many questions
had any right passage, and how many idle questions found anything (fewer is
better). Run from the repository root; it builds examples/volumina_ask.rs.
"""
import collections
import os
import subprocess
import sys

OFF = {"cleancode": "mechanical arts", "herbal": "herbs", "astronomy": "stars",
       "sicp": "mechanical arts", "behave": "love"}
books, verbose = sys.argv[1], "-v" in sys.argv
here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
gold, idle = collections.defaultdict(list), []
for line in open(os.path.join(here, "tools", "volumina_aurum.txt"), encoding="utf-8"):
    if line.startswith("#") or "|" not in line:
        continue
    b, q, r = line.rstrip("\n").split("|")
    if b == "idle":
        idle.append(q)
        continue
    pages = set()
    for part in r.split(","):
        a, _, c = part.partition("-")
        pages.update(range(int(a), int(c or a) + 1))
    gold[b].append((q, pages))


def ask(b, qs):
    out = subprocess.run(
        ["cargo", "run", "-q", "--example", "volumina_ask", "--", "3000", OFF[b],
         f"{os.path.join(books, b)}.txt:{OFF[b]}"],
        input="\n".join(qs) + "\n", capture_output=True, text=True, cwd=here).stdout
    res = []
    for line in out.splitlines():
        if line.strip().startswith("->"):
            t = line.split("->", 1)[1].strip()
            res.append([] if t == "nothing" else [int(x.rsplit(" ", 1)[1]) for x in t.split(" | ")])
    return res


tot = collections.Counter()
for b in OFF:
    qs = [q for q, _ in gold[b]]
    got = ask(b, qs + idle)
    c = collections.Counter()
    for (q, pages), g in zip(gold[b], got[:len(qs)]):
        k = "N" if not g else ("R" if g[0] in pages else "W")
        c[k] += 1
        c["any"] += bool(set(g) & pages)
        if verbose and k != "R":
            print(f"  {b} {k} {q} -> {g}")
    c["idle"] = sum(1 for g in got[len(qs):] if g)
    tot.update(c)
    print(f"{b:10} right {c['R']:2} wrong {c['W']:2} nothing {c['N']:2} | "
          f"any-right {c['any']:2} of {len(qs)} | idle finds {c['idle']}")
print(f"{'TOTAL':10} right {tot['R']:2} wrong {tot['W']:2} nothing {tot['N']:2} | "
      f"any-right {tot['any']:2} of {sum(len(v) for v in gold.values())} | idle finds {tot['idle']}")
