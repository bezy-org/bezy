use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct GameOfLifeState {
    pub grid: Vec<Vec<bool>>,
    pub width: usize,
    pub height: usize,
    pub generation: u32,
    pub last_update: Instant,
    pub update_interval: Duration,
    pub paused: bool,
}

impl GameOfLifeState {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::thread_rng();
        let grid = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| rng.gen_bool(0.3)) // 30% chance of being alive
                    .collect()
            })
            .collect();

        Self {
            grid,
            width,
            height,
            generation: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(125), // Update every 125ms (8 generations per second)
            paused: false,
        }
    }

    pub fn should_update(&self) -> bool {
        !self.paused && self.last_update.elapsed() >= self.update_interval
    }

    pub fn update(&mut self) {
        if !self.should_update() {
            return;
        }

        let mut new_grid = vec![vec![false; self.width]; self.height];

        for y in 0..self.height {
            for x in 0..self.width {
                let alive_neighbors = self.count_neighbors(x, y);
                let is_alive = self.grid[y][x];

                // Conway's Game of Life rules:
                // 1. Any live cell with 2-3 neighbors survives
                // 2. Any dead cell with exactly 3 neighbors becomes alive
                // 3. All other cells die or remain dead
                new_grid[y][x] = match (is_alive, alive_neighbors) {
                    (true, 2) | (true, 3) => true, // Survival
                    (false, 3) => true,            // Birth
                    _ => false,                    // Death or remains dead
                };
            }
        }

        self.grid = new_grid;
        self.generation += 1;
        self.last_update = Instant::now();
    }

    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0;

        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the cell itself
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                // Check bounds
                if nx >= 0 && ny >= 0 && (nx as usize) < self.width && (ny as usize) < self.height {
                    if self.grid[ny as usize][nx as usize] {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.width, self.height);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        if width != self.width || height != self.height {
            // Create a new grid with the new dimensions
            let mut new_grid = vec![vec![false; width]; height];

            // Copy over existing cells that fit in the new grid
            let copy_height = self.height.min(height);
            let copy_width = self.width.min(width);

            for y in 0..copy_height {
                for x in 0..copy_width {
                    new_grid[y][x] = self.grid[y][x];
                }
            }

            // If the new grid is larger, randomly populate the new cells
            if width > self.width || height > self.height {
                let mut rng = rand::thread_rng();
                for y in 0..height {
                    for x in 0..width {
                        // Only populate cells outside the copied area
                        if x >= self.width || y >= self.height {
                            new_grid[y][x] = rng.gen_bool(0.3);
                        }
                    }
                }
            }

            self.grid = new_grid;
            self.width = width;
            self.height = height;
        }
    }
}
