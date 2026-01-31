<!--
SPDX-FileCopyrightText: 2026 the Sandbox contributors
SPDX-License-Identifier: GPL-3.0-or-later
-->

# Spaceminer - Game Design Document

## Overview
A 2D space mining game where players pilot a ship through asteroid fields, extract ore, and deliver it to supply depots.

## Core Loop
```
Fly → Mine → Deliver → Upgrade → Repeat
```

## Player Ship

### Movement
- **Thrust-based**: WASD applies thrust in ship's local directions
- **Inertia**: Ship maintains velocity when not thrusting
- **Drag**: Light drag coefficient prevents infinite drift
- **No rotation** (initially): Ship always faces "up", WASD is screen-relative

### Controls
- `W` - Thrust up
- `S` - Thrust down
- `A` - Thrust left
- `D` - Thrust right
- `Space` - Mine (when near asteroid) [future]
- `E` - Interact (deposit ore) [future]

### Ship Stats (Future)
- Thrust power
- Max velocity
- Cargo capacity
- Mining speed
- Hull integrity

## Asteroids

### Ore Types (Future)
| Ore | Color | Value | Rarity |
|-----|-------|-------|--------|
| Iron | Gray | Low | Common |
| Copper | Orange | Medium | Uncommon |
| Gold | Yellow | High | Rare |
| Crystal | Blue | Very High | Very Rare |

### Mining Mechanic (Future)
- Approach asteroid within mining range
- Hold mining key to extract
- Progress bar fills
- Ore added to cargo when complete
- Asteroid depletes / respawns

## Supply Depot (Future)

### Functionality
- Safe zone (no asteroids spawn nearby)
- Deposit ore for credits
- Purchase upgrades
- Repair ship

### Visual
- Space station sprite
- Docking indicator when in range
- UI panel when docked

## World

### Current Implementation
- Infinite empty space
- Single ship
- Camera follows ship

### Future Implementation
- Procedural asteroid field
- Multiple depot locations
- Sector boundaries
- Minimap

## Future Ideas

### Enemies
- Space pirates that chase player
- Turrets on valuable asteroids
- Rival miners (AI competition)

### Upgrades
- Better engines (thrust/max speed)
- Larger cargo hold
- Mining laser (faster extraction)
- Shields
- Weapons

### Trading
- Different depots pay different prices
- Supply/demand fluctuation
- Trade routes

### Progression
- Unlock new ship types
- Reputation system
- Story missions

## Art Style
- Simple geometric shapes initially (rectangles, circles)
- Pixel art sprites later
- Dark space background with stars
- Glowing effects for thrusters, mining beam

## Audio (Future)
- Ambient space sounds
- Thruster hum
- Mining laser zap
- Ore collection ding
- Depot jingle
