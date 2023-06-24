slint::include_modules!();

struct LightsOutState{
    size: usize,
    activations: Vec<bool>,
}
impl LightsOutState{
    fn new(size: usize) -> Self {
        Self { size, activations: vec![false; size*size] }
    }
    fn resize(&mut self, size: usize){
        self.size = size;
        self.activations.resize(size*size, false);
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = LightsOut::new()?;
    let mut lights_out_state = LightsOutState::new(ui.get_button_field_size() as usize);
    ui.on_notify_tile_clicked(move |x, y| {
        let idx = lights_out_state.size * y as usize + x as usize;
        lights_out_state.activations[idx] = !lights_out_state.activations[idx];
        std::rc::Rc::new(slint::VecModel::from(light_buttons(lights_out_state.size, &lights_out_state.activations))).into()
    });
    ui.run()
}

fn light_buttons(size: usize, activations: &[bool]) -> Vec<bool>{
    let lights_system_matrix = build_light_system_matrix(size);
    let lights = (&lights_system_matrix) * (&SimpleBoolMatrix::new(activations.to_vec(), activations.len(), 1));
    lights.data
}

// Simple bool-matrix implementation to facilitate matrix-vector multiplication and aggregation of related information
// Assumes row-major layout
struct SimpleBoolMatrix{
    data: Vec<bool>,
    nrows: usize,
    ncols: usize,
}
impl SimpleBoolMatrix{
    fn new(data: Vec<bool>, nrows: usize, ncols: usize) -> Self{
        assert_eq!(data.len(), nrows*ncols);
        Self { data, nrows, ncols }
    }
}

impl std::ops::Add<&Self> for SimpleBoolMatrix{
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        assert!(self.nrows == rhs.nrows && self.ncols == rhs.ncols);
        for (left, right) in self.data.iter_mut().zip(rhs.data.iter()){
            *left ^= right;
        }
        self
    }
}

impl std::ops::Mul<Self> for &SimpleBoolMatrix{
    type Output = SimpleBoolMatrix;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.ncols, rhs.nrows);
        let n = self.ncols;
        let mut out = Vec::with_capacity(self.nrows * rhs.ncols);
        for i in 0..self.nrows{
            for j in 0..rhs.ncols{
                let self_ith_row_iter = self.data.iter().skip(i*self.ncols).take(n);
                let rhs_jth_col_iter = rhs.data.iter().skip(j).step_by(rhs.ncols).take(n);
                let count: usize = self_ith_row_iter.zip(rhs_jth_col_iter).map(|(rowval, colval)| {(*rowval && *colval) as usize }).sum();
                out.push((count % 2) == 1)
            }
        }
        SimpleBoolMatrix::new(out, self.nrows, rhs.ncols)
    }
}

// Build a matrix which models the lights out game as a linear equation modulo 2.
// https://mathworld.wolfram.com/LightsOutPuzzle.html
fn build_light_system_matrix(size: usize) -> SimpleBoolMatrix {
    let num_tiles = size*size;
    let num_system_matrix_entries = num_tiles * num_tiles;
    let mut matrix = vec![false; num_system_matrix_entries];

    for tile_idx in 0..num_tiles{
        let diag_idx = num_tiles * tile_idx + tile_idx;
        matrix[diag_idx] = true;
        if tile_idx < num_tiles - size {
            // Entry corresponds to some tile with another row below it
            matrix[diag_idx + size] = true;
        }
        if tile_idx >= size {
            // Entry corresponds to some tile with another row above it
            matrix[diag_idx - size] = true;
        }
        if (tile_idx % size) > 0{
            // Entry corresponds to some tile with another column to its left
            matrix[diag_idx-1] = true;
        } 
        if (tile_idx % size) < (size-1){
            // Entry corresponds to some tile with another column to its right
            matrix[diag_idx+1] = true; 
        }
    }
    SimpleBoolMatrix::new(matrix, num_tiles, num_tiles)
}