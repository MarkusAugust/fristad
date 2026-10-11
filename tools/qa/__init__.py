"""Kvalitetsportene (`docs/plan.md` del 3): rene funksjoner som måler en ting
(et teksturlag, et forbildeark, en modell) og gir en liste med funn. En port
som feiler, stopper løypa; tersklene står i koden med begrunnelse og
strammes, aldri slakkes, uten at alt godkjent kjøres på nytt.

    python3 -m tools.qa texture [lag …]     # P-T på teksturlagene
"""
