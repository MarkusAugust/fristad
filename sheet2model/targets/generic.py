"""Standardmålet: humanoid med 18 bein, ingen klipp, ingen slanking. Kopier denne for et nytt prosjekt."""
from .base import Target as _Base


class Target(_Base):
    name = "generic"
