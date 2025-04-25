use std::collections::hash_map::{HashMap};
use std::collections::HashSet;

/// Represents the type of a cell in the grid
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum CellType {
    Dead,
    Red,
    Blue,
}

impl CellType {
    /// Returns true if the cell is alive (red or blue)
    pub fn is_alive(&self) -> bool {
        match self {
            CellType::Dead => false,
            _ => true,
        }
    }
    
    /// Returns the cell type resulting from combining two cell types
    /// Used when calculating next state - if red and blue cells create neighbors, what happens?
    /// For now, we'll say red dominates (arbitrary choice)
    pub fn combine(a: CellType, b: CellType) -> CellType {
        match (a, b) {
            (CellType::Dead, other) | (other, CellType::Dead) => other,
            (CellType::Red, _) | (_, CellType::Red) => CellType::Red, // Red dominates
            (CellType::Blue, CellType::Blue) => CellType::Blue,
        }
    }
}

#[derive(PartialEq,Eq,Hash,Clone,Copy)]
pub struct Loc {
  pub row: i64,
  pub col: i64,
}

impl Loc {
  pub fn new(row: i64, col: i64) -> Self {
    Self {
      row,
      col,
    }
  }

  pub fn neighbors(&self) -> [Loc;8] {
    [
      Loc::new(self.row + 1, self.col + 1),
      Loc::new(self.row + 1, self.col - 1),
      Loc::new(self.row - 1, self.col + 1),
      Loc::new(self.row - 1, self.col - 1),
      Loc::new(self.row + 1, self.col    ),
      Loc::new(self.row    , self.col + 1),
      Loc::new(self.row - 1, self.col    ),
      Loc::new(self.row    , self.col - 1),
    ]
  }
}

pub struct World {
  buffer_1: HashMap<Loc, CellType>,
  buffer_2: HashMap<Loc, CellType>,
  using_buffer_1: bool,
}

impl World {

  pub fn new() -> World {
    Self {
      buffer_1: HashMap::new(),
      buffer_2: HashMap::new(),
      using_buffer_1: true,
    }
  }

  /**
   * Initialize from a configuration string. Assumes string is a grid of 
   * periods and asterisks (rows separated by line breaks), where asterisks
   * are "alive" cells and periods are dead cells.
   */
  pub fn from_configuration(data: &str, dead_char: char, alive_char: char) -> Result<Self,String> {
    let mut world = Self::new();

    let mut row = 0;
    let mut col = 0;

    for c in data.chars() {
      if c == dead_char {
        world.set(&Loc { row, col }, CellType::Dead);
        col += 1;
      } else if c == alive_char {
        world.set(&Loc { row, col }, CellType::Red); // Default to red for alive cells from config
        col += 1;
      } else if c == '\n' {
        row += 1;
        col = 0;
      } else if c == '\r' {
        // do nothing
      } else {
        return Err(format!("Invalid char '{}' at {}, {}", c, row, col));
      }
    }

    return Ok(world);
  }

  pub fn current_buffer(&self) -> &HashMap<Loc, CellType> {
    if self.using_buffer_1 { 
      &self.buffer_1 
    } else { 
      &self.buffer_2 
    }
  }

  fn current_buffer_mut(&mut self) -> &mut HashMap<Loc, CellType> {
      if self.using_buffer_1 {
          &mut self.buffer_1
      } else {
          &mut self.buffer_2
      }
  }

  fn next_buffer(&mut self) -> &mut HashMap<Loc, CellType> {
    if self.using_buffer_1 {
      &mut self.buffer_2
    } else { 
      &mut self.buffer_1 
    }
  }

  /**
   * Get cell type at a location in the world.
   */
  pub fn get(&self, loc: &Loc) -> CellType {
    *self.current_buffer().get(loc).unwrap_or(&CellType::Dead)
  }

  /**
   * Set cell type of a location in the world.
   * This updates the *next* buffer, used during the simulation step.
   */
  pub fn set(&mut self, loc: &Loc, cell_type: CellType) {
    let next_buffer = self.next_buffer();

    // If this location is already in the HashMap, set its value. Otherwise,
    // add it as a new entry to the HashMap.
    match next_buffer.get_mut(loc) {
      Some(val) => *val = cell_type,
      None => { next_buffer.insert(*loc, cell_type); }
    };

    if cell_type.is_alive() {
      // If this location is now alive, we need to add any of its neighbors not 
      // already in the HashMap, to it.
      for neighbor in loc.neighbors().iter() {
          // Use entry API for efficiency: only insert if the key doesn't exist.
          next_buffer.entry(*neighbor).or_insert(CellType::Dead);
      }
    }
  }

  /**
   * Set cell type of a location in the world *immediately* in the current buffer.
   * Used for direct user interaction (e.g., clicking).
   */
  pub fn set_cell_now(&mut self, loc: &Loc, cell_type: CellType) {
    let current_buffer = self.current_buffer_mut();

    // Set the clicked location to the specified cell type. Insert if not present.
    current_buffer.insert(*loc, cell_type);

    // Also ensure neighbors are in the map (as dead) so they are considered
    // in the next step and rendering.
    for neighbor in loc.neighbors().iter() {
        // Use entry API for efficiency: only insert if the key doesn't exist.
        current_buffer.entry(*neighbor).or_insert(CellType::Dead);
    }
  }

  /**
   * Set location alive with RED cell type (for backward compatibility).
   */
  pub fn set_alive_now(&mut self, loc: &Loc) {
    self.set_cell_now(loc, CellType::Red);
  }

  /**
   * Swaps the current and next buffers and clears the new next buffer.
   * Useful after initializing the world from a configuration.
   */
  pub fn swap_buffers_and_clear(&mut self) {
    // Toggle buffers
    self.using_buffer_1 = !self.using_buffer_1;
    // Clear the old buffer (which is now the next buffer)
    self.next_buffer().clear();
  }

  /**
   * One "tick" of the world.
   */
  pub fn step(&mut self) {
    let current_buffer = self.current_buffer();
    let mut candidates = HashSet::new();

    // Identify candidate cells: live cells and their neighbors
    for (loc, cell_type) in current_buffer.iter() {
      if cell_type.is_alive() {
        candidates.insert(*loc);
        for neighbor in loc.neighbors().iter() {
            candidates.insert(*neighbor);
        }
      }
    }

    // Calculate the next state for candidate cells and store results
    let mut next_states = Vec::new();
    for loc in candidates.iter() {
        let current_type = self.get(loc);
        let current_alive = current_type.is_alive();
        
        let neighbors = loc.neighbors();
        
        // Count alive neighbors by type
        let mut red_neighbors = 0;
        let mut blue_neighbors = 0;
        
        for neighbor in neighbors.iter() {
            match *current_buffer.get(neighbor).unwrap_or(&CellType::Dead) {
                CellType::Red => red_neighbors += 1,
                CellType::Blue => blue_neighbors += 1,
                CellType::Dead => {}
            }
        }
        
        let total_alive_neighbors = red_neighbors + blue_neighbors;
        
        // Apply Conway's Game of Life rules to determine if the cell lives
        let will_be_alive = if current_alive {
            // Live cell stays alive with 2 or 3 neighbors
            total_alive_neighbors == 2 || total_alive_neighbors == 3
        } else {
            // Dead cell becomes alive with exactly 3 neighbors
            total_alive_neighbors == 3
        };
        
        let next_type = if will_be_alive {
            // Cell will be alive, determine its type
            if current_alive {
                // If already alive, maintain its current type
                current_type
            } else {
                // If becoming alive, determine type based on neighbors
                if red_neighbors > blue_neighbors {
                    CellType::Red
                } else if blue_neighbors > red_neighbors {
                    CellType::Blue
                } else if red_neighbors > 0 {
                    // Equal numbers but at least one red
                    CellType::Red  // Red dominates (arbitrary)
                } else {
                    // Shouldn't happen if total is 3, but just in case
                    CellType::Blue
                }
            }
        } else {
            CellType::Dead
        };

        // Only store if the cell will be alive or if it was alive before
        if will_be_alive || current_alive {
             next_states.push((*loc, next_type));
        }
    }

    // Drop the immutable borrow of current_buffer here
    // Now apply the results to the next buffer
    for (loc, next_type) in next_states {
        self.set(&loc, next_type);
    }

    // Swap buffers and clear the new next buffer
    self.swap_buffers_and_clear();
  }
}

/**
 * Whether or not the supplied location is alive, based on the supplied buffer.
 */
fn is_alive(buffer: &HashMap<Loc, CellType>, loc: &Loc) -> bool {
  buffer.get(loc).map_or(false, |cell_type| cell_type.is_alive())
}
