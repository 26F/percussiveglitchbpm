# percussiveglitchbpm

Percussive glitch effect for music production which acts on beat triggered probability of a glitch event.
When a percussive glitch effect is triggered it triggers a cascade which has a probability of further recursing. This halfs
the beat and doubles it in repetitions: in a binary tree like probabilistic structure. 

It uses:
filename
beats per minute 
glitch beat 
probability of node recurse 
probability of initial cascade

usage:

./main input.wav 120 2 0.25 0.30

bpm of 120, glitch probability on every half note, probability of node recurse on glitch of 0.25 
and probability of initial glitch (causing recurse cascade) of 0.30

it won't work miracles.

You will more than likely want to splice with your original audio track to get the desired results.

Some software like MilkyTracker doesn't use standard definitions of bpm. You can use sites like beatsperminuteonline to get
what bpm for your track.

You can also test that your assumption about tempo is correct:
./main input.wav 120 2 0.25 1.0

(using a probability of always glitch puts (1.0) puts a click track over the song
so that you can test tempo.)

[test commit...]