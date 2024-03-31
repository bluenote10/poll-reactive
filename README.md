# poll-reactive

A minimalistic poll-based reactive library.

Instead of push-based reactivity, it opts for pull-based reactivity, i.e., components have to poll.
This mainly make sense for e.g. game engines where components have a "run once per frame" method anyway.
In return it comes with a number of simplifications like avoiding the need for `'static` closures, and a "natural" batching.
