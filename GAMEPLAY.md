# 🎮 SSH Poker Game - How to Play

## Getting Started

### 1. Connect to the Server
```bash
ssh -p 2222 demo@localhost
# Password: demo123
```

### 2. Navigate the Interface

When you connect, you'll see:

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                                                                              ║
║    ♠♥♦♣  WELCOME TO THE SSH POKER LOBBY  ♣♦♥♠                                ║
║                                                                              ║
║      ┌─────────────────────────────────────────────────────────────┐        ║
║      │                    AVAILABLE TABLES                         │        ║
║      └─────────────────────────────────────────────────────────────┘        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

Available Tables:
1. Main Table (2/6 players)
2. High Stakes (1/6 players)

Press number to join table, 'N' to create new table
Commands: (Q)uit
```

## Game Controls

### Lobby Controls
- **Numbers (1-9)**: Join an existing table
- **N**: Create a new table
- **Q**: Quit the game

### In-Game Controls
- **F**: Fold your hand
- **C**: Call (match current bet) or Check (if no bet)
- **R**: Raise (increase the bet)
- **A**: All-in (bet all your chips)
- **Q**: Quit the game

## Understanding the Table

### Table Layout
```
     ┌─────────┐                                       ┌─────────┐
     │ SEAT 4  │                 ┌─────────┐           │ SEAT 3  │
     │  AI-3   │                 │ SEAT 2  │           │  AI-2   │
     │ $850    │                 │  AI-1   │           │ $1200   │
     └─────────┘                 │ $950    │           └─────────┘
                                 └─────────┘

                           ╔═══════════════╗
                           ║   A♠ K♥ Q♦    ║  <- Community Cards
                           ║               ║
                           ╚═══════════════╝

                              💰 POT: $150

     ┌─────────┐                                       ┌─────────┐
     │ SEAT 5  │                 ┌─────────┐           │ SEAT 1  │
     │  AI-4   │                 │ SEAT 0  │           │  AI-0   │
     │ $600    │                 │  (YOU)  │           │ $1100   │
     └─────────┘                 │ $1000   │           └─────────┘
                                 │ A♥ K♠   │  <- Your hole cards
                                 └─────────┘
```

### Information Display
- **Your seat**: Always at the bottom center, marked "(YOU)"
- **Your cards**: Visible as actual cards (e.g., A♥ K♠)
- **Opponent cards**: Shown as face-down (🂠 🂠)
- **Community cards**: Revealed progressively through betting rounds
- **Pot amount**: Current total pot in the center
- **Player chips**: Each player's remaining chips
- **Current bet**: Amount each player has bet this round

### Game Phases
- **PRE-FLOP**: Only hole cards dealt, no community cards
- **FLOP**: First 3 community cards revealed
- **TURN**: 4th community card revealed  
- **RIVER**: 5th and final community card revealed
- **SHOWDOWN**: Players reveal cards to determine winner

## Gameplay Flow

### 1. Getting Seated
- Choose a table from the lobby
- You'll be assigned the next available seat
- Wait for other players (minimum 2 players to start)

### 2. Hand Begins
- Each player receives 2 hole cards
- Small blind and big blind are posted automatically
- Betting round begins with player after big blind

### 3. Betting Rounds
When it's your turn, you'll see:
```
Available Actions:
(F) Fold  |  (C) Call $20  |  (R) Raise  |  (A) All-In

Press the corresponding key to make your move!
```

### 4. Action Options

**Fold (F)**
- Give up your hand and forfeit any chips already bet
- You're out for the rest of this hand

**Call (C)**  
- Match the current highest bet
- If no bet has been made, this becomes "Check"

**Raise (R)**
- Increase the bet amount
- Forces other players to call your higher bet or fold

**All-In (A)**
- Bet all your remaining chips
- Cannot be raised beyond your chip count

### 5. Winning Hands (Texas Hold'em Rankings)
1. **Royal Flush**: A, K, Q, J, 10 all same suit
2. **Straight Flush**: 5 consecutive cards, same suit
3. **Four of a Kind**: 4 cards of same rank
4. **Full House**: 3 of a kind + pair
5. **Flush**: 5 cards of same suit
6. **Straight**: 5 consecutive cards
7. **Three of a Kind**: 3 cards of same rank
8. **Two Pair**: 2 pairs of different ranks
9. **One Pair**: 2 cards of same rank
10. **High Card**: Highest single card

## Tips for Success

### 1. Starting Hands
**Strong hands to play:**
- Pocket pairs (AA, KK, QQ, JJ, etc.)
- High suited cards (AK♠, AQ♠, KQ♠)
- High off-suit cards (AK, AQ, KQ)

**Weak hands to fold:**
- Low unsuited cards (72, 84, 95)
- Large gaps (J5, Q3, K2)

### 2. Position Matters
- **Late position** (seats 0-1): See others act first, play more hands
- **Early position** (seats 3-5): Act first, play tighter (fewer hands)

### 3. Reading the Board
- Look for flush draws (3+ same suit)
- Watch for straight possibilities
- Consider what hands could beat yours

### 4. Betting Strategy
- **Bet for value**: When you have a strong hand
- **Bluff occasionally**: Make opponents fold better hands
- **Fold when behind**: Don't chase unlikely draws

### 5. Managing AI Opponents
Each AI has different personalities:
- **Tight players**: Fold a lot, bet strong when they play
- **Loose players**: Play many hands, call frequently  
- **Aggressive players**: Bet and raise often
- **Conservative players**: Play very safely

## Troubleshooting

### Connection Issues
```bash
# If connection fails, try:
ssh -v -p 2222 demo@localhost

# Or check if server is running:
netstat -an | grep 2222
```

### Terminal Issues
- Use a terminal with ANSI color support
- Ensure terminal is at least 80x24 characters
- On Windows, use PowerShell, WSL, or PuTTY

### Game Issues
- Press **Q** to safely quit
- Server logs will show any errors
- Restart server if needed

## Advanced Features

### Creating Custom Tables
- Press **N** in lobby to create a new table
- Set custom blinds and player limits
- Name your table

### Statistics Tracking
- Game tracks your wins/losses
- Hand histories stored in database
- View your performance over time

Enjoy the game! 🎰♠♥♦♣