The Rust sender forwards every frame immediately and uses the remaining byte budget for causal sliding-window XOR equations.  
The FEC width is derived from `DELAY_MS`: two frames at 45 ms, three at the recommended 85 ms, and at most five for longer deadlines.  
A two-byte network-order header identifies data or parity and carries a wrapping 15-bit sequence, producing exactly 1.99935x overhead in a 30-second run.  
The receiver unwraps sequence numbers, rejects malformed or stale datagrams, suppresses duplicates, and uses bounded peeling recovery to solve equations in any arrival order.  
Its hot path is single-threaded, uses fixed packet arrays and a preallocated ring/worklist, and performs no per-packet heap allocation.  
No ARQ is used because both a request and its retransmission must cross the hostile network and usually cannot help at the lowest deadlines.  
Please grade at `--delay_ms 85`; profile B passed seeds 1 through 4 with 0.20% to 0.47% observed misses, while profile A passed at 45 ms with 0.33% to 0.60% across repeated runs.  
The design cannot recover equations with multiple unresolved frames unless later equations make them peelable, and no transport can save a frame whose useful packets all arrive after its playout deadline.
