# SSH Poker Game - Architecture Diagrams

This document provides visual representations of the system architecture, component structure, and data flow for the SSH Poker Game.

## 1. System Architecture Overview

The system follows a layered architecture with clear separation of concerns:

```mermaid
graph TB
    subgraph "Client Layer"
        SSH[SSH Client<br/>Terminal]
    end
    
    subgraph "Network Layer"
        SSHD[SSH Server<br/>russh:2222]
    end
    
    subgraph "Application Layer"
        SM[Session Manager<br/>Player Sessions & Tables]
        CH[SSH Handler<br/>TUI Bridge]
        TUI[Casino TUI<br/>poker-tui]
        GE[Game Engine<br/>poker-engine]
        AI[AI Bot<br/>ai-bot]
    end
    
    subgraph "Data Layer"
        DS[Data Store<br/>SQLite DB]
        HM[Metrics<br/>Prometheus/Datadog]
    end
    
    SSH -->|SSH Protocol| SSHD
    SSHD -->|PTY/Channel| CH
    CH <-->|Events/Updates| TUI
    CH <-->|Auth/Sessions| SM
    SM <-->|Game State| GE
    SM <-->|User Data| DS
    GE <-->|Bot Actions| AI
    TUI -->|Display| SSH
    SM -->|Metrics| HM
    
    style SSH fill:#e1f5fe
    style SSHD fill:#4fc3f7
    style TUI fill:#ffd54f
    style GE fill:#81c784
    style DS fill:#ce93d8
    style AI fill:#ffb74d
```

### Key Components:
- **Client Layer**: Any SSH-compatible terminal client
- **Network Layer**: SSH server handling secure connections
- **Application Layer**: Core game logic and UI management
- **Data Layer**: Persistent storage and monitoring

## 2. Component Architecture

Detailed view of the internal structure of each crate:

```mermaid
graph LR
    subgraph "SSH Server Crate"
        Main[main.rs<br/>Entry Point]
        Auth[auth.rs<br/>Authentication]
        SecAuth[secure_auth.rs<br/>Password/Key Auth]
        Session[session.rs<br/>Session Management]
        SshH[ssh_handler.rs<br/>TUI Integration]
    end
    
    subgraph "Poker TUI Crate"
        App[app.rs<br/>App State]
        Views[views.rs<br/>Lobby/Game Views]
        Comp[components.rs<br/>UI Components]
        Events[events.rs<br/>Event System]
        Themes[themes.rs<br/>Casino Styling]
    end
    
    subgraph "Poker Engine Crate"
        Game[game.rs<br/>Game State]
        FSM[fsm.rs<br/>State Machine]
        Card[card.rs<br/>Card/Deck]
        Hand[hand.rs<br/>Hand Evaluation]
        Betting[betting.rs<br/>Betting Logic]
        Player[player.rs<br/>Player Model]
    end
    
    subgraph "Data Store Crate"
        DB[Database<br/>Connection Pool]
        Models[models.rs<br/>User/Game Models]
        Ops[operations.rs<br/>CRUD Operations]
        Schema[schema.rs<br/>DB Schema]
    end
    
    subgraph "AI Bot Crate"
        Strategy[strategy.rs<br/>AI Strategies]
        Eval[evaluator.rs<br/>Hand Evaluation]
        Person[personality.rs<br/>Bot Personalities]
    end
    
    Main --> Auth
    Auth --> SecAuth
    Main --> Session
    Main --> SshH
    SshH --> App
    App --> Views
    Views --> Comp
    App --> Events
    Session --> Game
    Game --> FSM
    Game --> Betting
    Game --> AI
    Session --> DB
    
    style Main fill:#ff6b6b
    style App fill:#4ecdc4
    style Game fill:#95e1d3
    style DB fill:#f3a683
    style Strategy fill:#778beb
```

### Crate Responsibilities:
- **ssh-server**: Network handling, authentication, and session management
- **poker-tui**: Beautiful casino-themed terminal interface
- **poker-engine**: Core game logic and state management
- **data-store**: Database operations and user management
- **ai-bot**: Intelligent computer opponents

## 3. Data Flow Diagram

Sequence diagram showing the flow of data through the system during a typical game session:

```mermaid
sequenceDiagram
    participant C as SSH Client
    participant S as SSH Server
    participant A as Auth Service
    participant SM as Session Manager
    participant TUI as Casino TUI
    participant GE as Game Engine
    participant DB as Database
    participant AI as AI Bot
    
    C->>S: SSH Connection Request
    S->>A: Authenticate User
    A->>DB: Verify Credentials
    DB-->>A: User Data
    A-->>S: Auth Success
    S->>SM: Create Session
    SM->>DB: Store Session
    S->>TUI: Initialize TUI
    TUI-->>C: Display Lobby
    
    rect rgb(200, 230, 200)
        Note over C,TUI: Player Joins Table
        C->>TUI: Select Table
        TUI->>SM: Join Table Request
        SM->>GE: Add Player to Game
        GE-->>SM: Game State Updated
        SM-->>TUI: Update Display
        TUI-->>C: Show Game View
    end
    
    rect rgb(230, 200, 200)
        Note over C,AI: Game Play Loop
        loop Each Betting Round
            GE->>TUI: Request Player Action
            TUI-->>C: Show Options
            C->>TUI: Player Action
            TUI->>SM: Process Action
            SM->>GE: Update Game State
            
            alt AI Bot Turn
                GE->>AI: Request Bot Action
                AI->>AI: Evaluate Strategy
                AI-->>GE: Bot Action
            end
            
            GE->>DB: Save Game State
            GE-->>SM: Broadcast Updates
            SM-->>TUI: Update All Players
            TUI-->>C: Refresh Display
        end
    end
    
    rect rgb(200, 200, 230)
        Note over C,DB: Disconnect/Cleanup
        C->>S: Disconnect
        S->>SM: Remove Session
        SM->>GE: Remove Player
        SM->>DB: Update Records
        SM-->>S: Cleanup Complete
    end
```

### Key Data Flows:
1. **Authentication Flow**: SSH → Auth Service → Database → Session Creation
2. **Game Join Flow**: TUI → Session Manager → Game Engine → State Update
3. **Game Play Flow**: Player Input → Game Engine → State Updates → Broadcast to All Players
4. **AI Integration**: Game Engine → AI Bot → Strategy Evaluation → Action Response
5. **Cleanup Flow**: Disconnect → Session Removal → Game State Update → Database Update

## Architecture Principles

### 1. Separation of Concerns
Each crate has a specific responsibility:
- Network handling is isolated in ssh-server
- UI logic is contained in poker-tui
- Game rules are enforced by poker-engine
- Data persistence is managed by data-store

### 2. Event-Driven Architecture
The system uses an event-driven approach:
- User inputs generate events
- Game state changes trigger updates
- All clients receive real-time updates

### 3. Security by Design
- SSH provides encrypted communication
- Authentication happens before any game access
- Session management prevents unauthorized access
- Game state is server-authoritative to prevent cheating

### 4. Scalability Considerations
- Session Manager can handle multiple concurrent games
- Each table is independent for horizontal scaling
- Database connection pooling for efficient resource usage
- Async/await pattern for handling concurrent connections

## Future Architecture Enhancements

1. **Microservices**: Split game engine into separate service for better scaling
2. **Message Queue**: Add Redis/RabbitMQ for event distribution
3. **Load Balancer**: Add HAProxy for distributing SSH connections
4. **Caching Layer**: Add Redis for session and game state caching
5. **API Gateway**: RESTful API for web/mobile clients