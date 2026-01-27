# Project Context

> Generated: 2026-01-27 18:15 UTC

## chess-analyzer

**Language**: Rust

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| Use shakmaty for chess logic | Pure Rust, fast, well documented |
| Use pgn-reader for PGN parsing | Streaming parser, memory efficient |
| Stockfish via subprocess | Standard UCI protocol approach |

## Todo

- [x] Implement SAN to UCI conversion
- [ ] Add move-by-move analysis with mistake detection
- [ ] Integrate Lichess API
- [ ] Add puzzle generation from blunders
- [ ] Build game report generator

## Known Bugs

None currently tracked.

## Recent Progress

- **01/27**: Fixed SAN to UCI conversion - game analysis now works
- **01/15**: Goal: Analyze Lichess/Chess.com games, find blunders, generate personalized puzzles
- **01/15**: Initial codebase review with Claude
