# Tron Coding Challenge Description

Everyone will make their own AI that will play the game tron. They will do this by creating their own type that implements the Bot trait.
Each participant will make a module that is a submodule to the players module. I can either copy paste modules in or they can be pull requested.
No changes outside your own module are permitted besides renaming your own module.

See src/players/example_bot.rs for an example, or src/players/bot_template.rs for starter code. You can also ask me any questions you like so I can update this document to better help others. Or even PR changes to this doc.

WARNING! Example bot heavily uses the iterator feature of rust which may not be intuitive to first timers. I can work on making another example which is easier for those who arent familiar with iterators.

## Some rules of the game to keep in mind

- There are two players, X and O. Player O starts to the left and player X starts to the right. Your bot should be prepared to be either player.
- The grid is 21 by 21 with players starting adjacent to the center of the grid.
- Each cell of the grid can either be the head of either player, the tail of either player, or empty
- If your bots head ever collides with a tail, or the edge of the grid, or another players head, you lose. If both players do this at the same time it is a draw. (Colliding here means your bot inputs a direction that would cause an overlap)
- Your bot cannot require human input after the program starts executing
- Your bot will go against every other bot 6 times (3 times as player O, 3 times as player X). Winning gives you 1 point, losing gives you -1 points, and drawing gives you -0.5 points.

## Your bot must implement the bot trait

The new function just is for setting up your type
The next_action function is the core of your bot. This function takes in the state of the game, and returns what direction the bot should move in (THIS IS YOUR BOTS ENTIRE BRAIN!)
If your bot ever causes the program to stack overflow / crash / panic during a round, then your bot loses that round.
You can ask me to add an external library and I might allow it, but otherwise no external libraries are allowed.

If you sudo rm -rf my computer I will be angry with you.

## Some functions or types that might be a good place to start looking

### Types

- `GridPosition` - Represents a position that is in bounds in the grid
- `Direction` - Represents either up, down, left or right. (PositiveY, NegativeY, NegativeX, PositiveX respectively) (positive y is up, positive x is right)
- `Grid` - Represents the state of a grid and all the cells in it
- `GridCell` - represents a single cell of the grid. Either empty, a players tail, or a players head. It also contains the direction the player was moving when they left that tail. (does not store its own position)
- `GameState` - represents the current gamestate, containing the current grid and all previous grids. As well as weather or not the game is ended.
- `PlayerId` - Either player X or player O

### Functions

- `game_state.current_grid()` returns the current grid (`&Grid`)
- `grid.get_cell(position)` returns the cell (`&GridCell`) at a position (so does `position.get_cell(grid)`)
- `grid.player_head_positions()` returns a tuple of (Player O's position, Player X's position) (`(GridPosition, GridPosition)`)
- `cell.is_empty()` returns true if the cell is empty
- `cell.is_players_head(player)` returns true if the cell represents the given players head
- `game_state.current_time()` returns the frame number (`usize`) this game is up to

More complex functions ---

- `position.after_moved(direction)` returns a position adjacent to the input position in a given direction (`Option<GridPosition>`)
- `GridPosition::iter_positions()` returns an iterator over all positions
- `game_state.grid_history()` returns an iterator over all grids in the history in order since the start of the game

If there is some function I obviously should implement in the engine code that hasn't been implemented feel free to add it and make a PR, but no promises ill add it (I am not adding a call chatgpt api function to the engine)
