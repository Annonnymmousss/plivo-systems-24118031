# Experiment Run Log

All experiments used the unmodified supplied harness, Python 3.13.0, and Rust 1.95.0 on macOS. The reported overhead includes every media and feedback byte accepted by the relay. The final protocol emits no feedback, and its exact 30-second overhead is `479844 / 240000 = 1.99935x`.

| Implementation | Profile | Seed | Delay | Misses | Miss rate | Overhead | Change and reason |
|---|---:|---:|---:|---:|---:|---:|---|
| Naive C baseline | A | 1 | 40 ms | 330 / 1500 | 22.00% | 1.02x | Measured the supplied forwarding baseline; every network loss becomes a playout miss. |
| Naive C baseline | B | 1 | 80 ms | 217 / 1500 | 14.47% | 1.02x | Confirmed that forwarding alone is invalid under higher loss and jitter. |
| Adjacent XOR prototype | A | 1 | 45 ms | 11 / 1500 | 0.73% | 1.99935x | Sent systematic data plus two-frame causal parity to repair isolated loss. |
| Adjacent XOR prototype | B | 1 | 85 ms | 6 / 1500 | 0.40% | 1.99935x | First valid moderate-profile candidate. |
| Adjacent XOR prototype | B | 2 | 85 ms | 5 / 1500 | 0.33% | 1.99935x | Repeated the candidate against another impairment sequence. |
| Adjacent XOR prototype | B | 3 | 85 ms | 6 / 1500 | 0.40% | 1.99935x | Repeated the candidate against another impairment sequence. |
| Adjacent XOR prototype | B | 4 | 85 ms | 2 / 1500 | 0.13% | 1.99935x | Established a four-seed mean of 0.32%. |
| Distance-two experiment (rejected) | B | 1 | 85 ms | 9 / 1500 | 0.60% | 1.99935x | Interleaved parity separated burst losses but weakened recovery when an intermediate frame was unavailable. |
| Distance-two experiment (rejected) | B | 2 | 85 ms | 3 / 1500 | 0.20% | 1.99935x | Retested before deciding; the cross-seed worst case still regressed. |
| Distance-two experiment (rejected) | B | 3 | 85 ms | 6 / 1500 | 0.40% | 1.99935x | No consistent improvement over adjacent XOR. |
| Distance-two experiment (rejected) | B | 4 | 85 ms | 7 / 1500 | 0.47% | 1.99935x | Rejected rather than retaining complexity without measured value. |
| Deadline-aware sliding XOR (final) | A | 1 | 45 ms | 5 / 1500 | 0.33% | 1.99935x | Derived a two-frame equation width from the short deadline and replaced a magic parity period with a byte-credit budget. |
| Deadline-aware sliding XOR (final) | B | 1 | 85 ms | 5 / 1500 | 0.33% | 1.99935x | Used three-frame causal equations and bounded peeling recovery. |
| Deadline-aware sliding XOR (final) | B | 2 | 85 ms | 3 / 1500 | 0.20% | 1.99935x | Confirmed the improvement on a second impairment sequence. |
| Deadline-aware sliding XOR (final) | B | 3 | 85 ms | 5 / 1500 | 0.33% | 1.99935x | Confirmed the worst observed final miss rate remained far below 1%. |
| Deadline-aware sliding XOR (final) | B | 4 | 85 ms | 3 / 1500 | 0.20% | 1.99935x | Final four-seed mean: 0.27%, improved from 0.32%. |
| Deadline-aware sliding XOR (final) | Synthetic burst + spikes | 11 | 170 ms | 5 / 1500 | 0.33% | 1.99935x | Exercised correlated loss, 80 ms ordinary jitter, extra 80 ms spikes, duplication, and five-frame equations. |
| Adjacent XOR prototype | Synthetic heavy reorder | 17 | 230 ms | 0 / 1500 | 0.00% | 1.99935x | Exercised 20% duplication, 3% loss, 100 ms jitter, and 120 ms spikes before the final FEC generalization. |
| Clean audited final build | A | 1 | 45 ms | 9 / 1500 | 0.60% | 1.99935x | Rebuilt from `make clean`, restored byte-identical supplied files, and reran the complete grader path; valid. |
| Clean audited final build | B | 1 | 85 ms | 7 / 1500 | 0.47% | 1.99935x | Repeated the complete grader path on the recommended delay after all code and report changes; valid. |

Small same-seed changes near a deadline can vary with host scheduling, but the final margins remain well below the 1% cap. The synthetic profiles were temporary local tests and are not part of the submission.

## Validation commands

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
make clean && make
python3 run.py --profile profiles/A.json --seed 1 --delay_ms 45
python3 run.py --profile profiles/B.json --seed 1 --delay_ms 85
```
