# Flaky Network Transport

## Native build and run

Build the optimized Rust sender and receiver:

```sh
make
```

Run the recommended grading configuration:

```sh
python3 run.py --profile profiles/B.json --seed 1 --delay_ms 85
```

Use **85 ms** as the assignment grading delay.

## Docker build and run

Build the image and run the default profile:

```sh
make docker
```

Docker defaults to 90 ms to allow for container scheduling overhead. Parameters can be overridden:

```sh
make docker PROFILE=profiles/A.json SEED=1 DELAY_MS=45 DURATION_SECONDS=30
```

Other Docker targets:

```sh
make docker-build
make docker-clean
```
