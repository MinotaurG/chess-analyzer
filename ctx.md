# Chess Analyzer

> Last updated: 2026-01-27

## Vision

A chess improvement app that analyzes your games, identifies weakness patterns, and helps drill them away through spaced repetition.

## Target Users

- Primary: Players learning fundamentals (800-1400 ELO)
- Secondary: Intermediate players (1400-1800) wanting structured improvement

## Platform Strategy

| Phase | Platform | Analysis | Timeline |
|-------|----------|----------|----------|
| 1 | Web (Rust + htmx) | Lichess Cloud Eval | Weeks 1-4 |
| 2 | Web (Polish) | + optional WASM Stockfish | Weeks 5-8 |
| 3 | Mobile (Flutter) | Cloud Eval only | Weeks 9-12 |

## Core Features (MVP)

### 1. Game Sync
- Connect Lichess account (OAuth)
- Import all historical games
- Store locally (SQLite)
- Future: Chess.com, manual PGN import

### 2. Pattern Detection
Analyze games to identify recurring mistakes:

**Tactical:** Hanging pieces, missed forks/pins/skewers, back rank, discovered attacks, trapped pieces

**Positional:** Bad bishops, weak squares, pawn structure errors, king safety

**Phase-based:** Opening inaccuracies, middlegame planning, endgame technique

**Psychological:** Time trouble blunders, tilt detection, momentum shifts

### 3. Weakness Report
- Aggregate analysis across all games
- Rank weaknesses by frequency and cost (centipawn loss)
- Track improvement over time

### 4. Drills (Phase 2+)
- Spaced repetition puzzles from YOUR mistakes
- Bot that plays your problem openings
- Targeted tactical exercises

## Technical Architecture
┌─────────────────────────────────────────┐
│ Web Browser │
│ ┌─────────────────────────────────┐ │
│ │ htmx + Server Templates │ │
│ └──────────────┬──────────────────┘ │
└─────────────────┼───────────────────────┘
│ HTTP
┌─────────────────▼───────────────────────┐
│ Axum Web Server │
│ ┌─────────────────────────────────┐ │
│ │ Route Handlers │ │
│ └──────────────┬──────────────────┘ │
│ ┌──────────────▼──────────────────┐ │
│ │ Core Library │ │
│ │ ┌─────────┐ ┌───────────────┐ │ │
│ │ │ Lichess │ │ Analysis │ │ │
│ │ │ API │ │ Engine │ │ │
│ │ └─────────┘ └───────────────┘ │ │
│ │ ┌─────────┐ ┌───────────────┐ │ │
│ │ │ Pattern │ │ SQLite │ │ │
│ │ │Detector │ │ Storage │ │ │
│ │ └─────────┘ └───────────────┘ │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────────┘
│
▼
┌─────────────────┐
│ Lichess API │
│ - Games export │
│ - Cloud eval │
│ - OAuth │
└─────────────────┘



## Data Models

### Game
id
lichess_id
white_username
black_username
white_elo
black_elo
result
time_control
opening_name
opening_eco
pgn
moves (JSON array of UCI)
analyzed (bool)
analysis_data (JSON)
created_at



### Pattern
id
game_id
move_number
pattern_type (enum)
subtype
severity (blunder/mistake/inaccuracy)
centipawn_loss
position_fen
description
created_at



### User
id
lichess_username
lichess_token (encrypted)
games_synced_at
settings (JSON)
created_at



## API Endpoints (Draft)
GET / # Dashboard
GET /auth/lichess # OAuth redirect
GET /auth/callback # OAuth callback
POST /games/sync # Fetch games from Lichess
GET /games # List games
GET /games/:id # Game detail with analysis
GET /patterns # Weakness report
GET /patterns/:type # Specific pattern breakdown
GET /drill/:pattern_type # Start drill session
POST /drill/:id/answer # Submit drill answer



## Dependencies

### Core
- shakmaty: Chess logic
- pgn-reader: PGN parsing
- reqwest: HTTP client
- rusqlite: SQLite
- serde/serde_json: Serialization

### Web
- axum: Web framework
- askama: Templates
- tower: Middleware
- tokio: Async runtime

## Todo

### Phase 1 - Week 1-2
- [ ] Restructure to workspace (core + web crates)
- [ ] Lichess API client (game export)
- [ ] SQLite schema and storage layer
- [ ] Basic web server with dashboard template
- [ ] Game list view

### Phase 1 - Week 3-4
- [ ] Lichess OAuth integration
- [ ] Cloud eval integration
- [ ] Pattern detection engine (basic)
- [ ] Weakness report view
- [ ] Game detail with move-by-move analysis

### Phase 2
- [ ] WASM Stockfish option
- [ ] Drill system
- [ ] Spaced repetition logic
- [ ] Progress tracking

### Phase 3
- [ ] Flutter mobile app
- [ ] Offline storage
- [ ] Push notifications

## Known Issues

None currently.

## Recent Progress

- **01/27**: Defined full project scope and architecture
- **01/27**: Fixed SAN to UCI conversion
- **01/15**: Initial project setup
