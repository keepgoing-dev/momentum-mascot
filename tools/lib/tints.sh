# The per-state lighting, shared by the room compositor and the layer compositor.
#
# Sourced rather than duplicated: a built mascot's layers are pre-tinted at build time, so
# these numbers have to be the same on both sides or the character will not match its room.

TINT_COLOUR="#3050a0"
TINT_DOZING=10
TINT_ASLEEP=34
COMEBACK_MODULATE="113,120"   # brightness,saturation
