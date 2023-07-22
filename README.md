## Usage
cargo run > README.md
## Five Node Single Region
Client Region | Destination | Leader Region | Response Time |
------------- | ----------- | ------------- | ------------- |
A | Leader | A | 20ms
A | Follower (A) | A | 30ms

Strategy | Min Response Time | Max Response Time | Mean Response Time
-------- | ----------------- | ----------------- | ------------------
Leader | 20ms | 20ms | 20ms
Local | 20ms | 30ms | 28ms
Global | 20ms | 30ms | 28ms

## Nine Node Multi Region
Client Region | Destination | Leader Region | Response Time |
------------- | ----------- | ------------- | ------------- |
A | Leader | A | 130ms
A | Leader | B | 160ms
A | Leader | C | 160ms
A | Follower (A) | A | 140ms
A | Follower (A) | B | 170ms
A | Follower (A) | C | 170ms
A | Follower (B) | A | 360ms
A | Follower (B) | B | 170ms
A | Follower (B) | C | 200ms
A | Follower (C) | A | 360ms
A | Follower (C) | B | 200ms
A | Follower (C) | C | 170ms
B | Leader | A | 240ms
B | Leader | B | 50ms
B | Leader | C | 80ms
B | Follower (A) | A | 250ms
B | Follower (A) | B | 280ms
B | Follower (A) | C | 280ms
B | Follower (B) | A | 250ms
B | Follower (B) | B | 60ms
B | Follower (B) | C | 90ms
B | Follower (C) | A | 280ms
B | Follower (C) | B | 120ms
B | Follower (C) | C | 90ms
C | Leader | A | 240ms
C | Leader | B | 80ms
C | Leader | C | 50ms
C | Follower (A) | A | 250ms
C | Follower (A) | B | 280ms
C | Follower (A) | C | 280ms
C | Follower (B) | A | 280ms
C | Follower (B) | B | 90ms
C | Follower (B) | C | 120ms
C | Follower (C) | A | 250ms
C | Follower (C) | B | 90ms
C | Follower (C) | C | 60ms

Strategy | Min Response Time | Max Response Time | Mean Response Time
-------- | ----------------- | ----------------- | ------------------
Leader | 50ms | 240ms | 132ms
Local | 50ms | 250ms | 141ms
Global | 50ms | 360ms | 196ms

