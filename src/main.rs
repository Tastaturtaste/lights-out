use std::{rc::Rc, cell::{RefCell, Cell}};

slint::include_modules!();

struct LightsOutState{
    size: usize,
    activations: Vec<bool>,
    lights: Vec<bool>,
    switch_to_solve: Vec<bool>,
}
impl LightsOutState{
    fn new(size: usize) -> Self {
        Self { size, activations: vec![false; size*size], lights: vec![false; size*size], switch_to_solve: vec![false; size*size]}
    }
    fn resize(&mut self, size: usize){
        self.size = size;
        self.deactivate_all();
    }
    fn deactivate_all(&mut self){
        let n = self.size*self.size;
        self.activations = vec![false; n];
        self.lights = vec![false; n];
        self.switch_to_solve = vec![false; n];
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = LightsOut::new()?;
    let lights_out_state = Rc::new(RefCell::new(LightsOutState::new(ui.get_button_field_size() as usize)));
    ui.on_notify_tile_clicked({
        let ui_handle = ui.as_weak();
        let state_ref = lights_out_state.clone();
        move |x, y| {
            let ui = ui_handle.unwrap();
            let mut lights_out_state = state_ref.borrow_mut();
            let idx = lights_out_state.size * y as usize + x as usize;
            lights_out_state.activations[idx] = !lights_out_state.activations[idx];
            lights_out_state.lights = light_buttons(lights_out_state.size, &lights_out_state.activations);
            lights_out_state.switch_to_solve[idx] = false;
            ui.set_button_field_lights(std::rc::Rc::new(slint::VecModel::from(lights_out_state.lights.clone())).into());
            ui.set_button_field_switch_to_solve(Rc::new(slint::VecModel::from(lights_out_state.switch_to_solve.clone())).into());
        }
    });
    ui.on_notify_reset_clicked({
        let ui_handle = ui.as_weak();
        let state_ref = lights_out_state.clone();
        move ||{
            let ui = ui_handle.unwrap();
            let mut lights_out_state = state_ref.borrow_mut();
            lights_out_state.deactivate_all();
            ui.set_button_field_lights(Rc::new(slint::VecModel::from(lights_out_state.lights.clone())).into());
            ui.set_button_field_switch_to_solve(Rc::new(slint::VecModel::from(lights_out_state.switch_to_solve.clone())).into())
        }
    });
    ui.on_notify_resize_request({
        let ui_handle = ui.as_weak();
        let state_ref = lights_out_state.clone();
        move |user_request|{
            let ui = ui_handle.unwrap();
            let mut lights_out_state = state_ref.borrow_mut();
            let validated_size = if let Ok(validated_size) = user_request.parse::<usize>() {
                validated_size
            }else{
                return
            };
            lights_out_state.resize(validated_size);
            ui.set_button_field_size(validated_size as i32);
            ui.set_button_field_lights(Rc::new(slint::VecModel::from(lights_out_state.lights.clone())).into());
            ui.set_button_field_switch_to_solve(Rc::new(slint::VecModel::from(lights_out_state.switch_to_solve.clone())).into());
        }
    });
    ui.on_notify_solve_clicked({
        let ui_handle = ui.as_weak();
        let state_ref = lights_out_state;
        move ||{
            let ui = ui_handle.unwrap();
            let mut lights_out_state = state_ref.borrow_mut();
            let missing_lights = light_buttons(lights_out_state.size, &lights_out_state.activations).into_iter().map(|light_on| !light_on).collect();
            let system_matrix = build_light_system_matrix(lights_out_state.size);
            lights_out_state.switch_to_solve = solve_switch_system(system_matrix, missing_lights);
            ui.set_button_field_switch_to_solve(Rc::new(slint::VecModel::from(lights_out_state.switch_to_solve.clone())).into());
        }
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
#[derive(Clone)]
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
    fn get_mut_rows(&mut self, selected_row_indices: &[usize]) -> Vec<&mut [bool]>{
        let mut rows = Vec::with_capacity(selected_row_indices.len());
        // Sorted indices into the selection indices so progressive splits are easy
        let sorted_selected_row_indices_indices = {
            let mut sorted_selection_indices: Vec<_> = (0..selected_row_indices.len()).collect();
            sorted_selection_indices.sort_by_key(|&i| &selected_row_indices[i]);
            sorted_selection_indices
        };
        // Setup split slices
        let mut remaining_data = &mut self.data[..];
        let mut used_rows = 0;
        for &sorted_selected_row_indices_idx in sorted_selected_row_indices_indices.iter(){
            let row_idx = selected_row_indices[sorted_selected_row_indices_idx];
            (_, remaining_data) = remaining_data.split_at_mut((row_idx - used_rows) * self.ncols);
            let row;
            (row, remaining_data) = remaining_data.split_at_mut(self.ncols);
            rows.push(row);
            used_rows = row_idx+1;
        }
        // Reorder the row slices to the originally requested order
        let rows = Cell::from_mut(rows.as_mut_slice()).as_slice_of_cells();
        sorted_selected_row_indices_indices.into_iter().map(|i| rows[i].take()).collect()
    }
}

impl std::ops::Add<&Self> for SimpleBoolMatrix{
    type Output = Self;

    fn add(mut self, rhs: &Self) -> Self::Output {
        assert!(self.nrows == rhs.nrows && self.ncols == rhs.ncols);
        for (left, right) in self.data.iter_mut().zip(rhs.data.iter()){
            *left = (Switch(*left) + Switch(*right)).into();
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
                let switch = self_ith_row_iter.zip(rhs_jth_col_iter).map(|(rowval, colval)| {Switch(*rowval) *  Switch(*colval) }).reduce(|acc, e| acc + e);
                if let Some(switch) = switch{
                    out.push(switch.into())
                }else{
                    continue;
                }
            }
        }
        SimpleBoolMatrix::new(out, self.nrows, rhs.ncols)
    }
}


// Implements the logical operations for operating on logical switches
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Switch(bool);
impl std::ops::Add for Switch{
    type Output=Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::Sub for Switch{
    type Output=Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::Mul for Switch{
    type Output=Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 && rhs.0)
    }
}
impl std::convert::From<Switch> for bool {
    fn from(value: Switch) -> Self {
        value.0
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

// Calculates the switches to use according to A to get activations in b
#[allow(non_snake_case)]
fn solve_switch_system(mut A: SimpleBoolMatrix, mut b: Vec<bool>) -> Vec<bool> {
    let row_indices: Vec<_> = (0..A.nrows).collect();
    let mut A_rows = A.get_mut_rows(&row_indices);
    let A_rows = Cell::from_mut(A_rows.as_mut_slice()).as_slice_of_cells();
    // Clear lower triangular to bring A in reduced row echolon form
    for selected_idx in 0..A_rows.len(){
        let selected_row = A_rows[selected_idx].take();
        // If the current row has no entry on the diagonal, swap it with a row that has
        if !selected_row[selected_idx]{
            for row_idx in (selected_idx+1)..A_rows.len(){
                let potential_swap_row = A_rows[row_idx].take();
                let mut break_flag = false;
                if potential_swap_row[selected_idx]{
                    selected_row.swap_with_slice(potential_swap_row);
                    b.swap(selected_idx, row_idx);
                    break_flag = true;
                }
                A_rows[row_idx].set(potential_swap_row);
                if break_flag{
                    break;
                }
            }
        }
        for reduced_idx in (selected_idx+1)..A_rows.len(){
            let reduced_row = A_rows[reduced_idx].take();
            if reduced_row[selected_idx] {
                (*reduced_row).iter_mut().zip(selected_row.iter()).for_each(|(reduced_val, &selected_val)| *reduced_val = (Switch(*reduced_val) - Switch(selected_val)).into());
                b[reduced_idx] = (Switch(b[reduced_idx]) - Switch(b[selected_idx])).into();
            }
            A_rows[reduced_idx].set(reduced_row);
        }
        A_rows[selected_idx].set(selected_row);
    }

    // Resubstitute bottom to top to clear upper triangular
    for selected_idx in (0..A_rows.len()).rev() {
        let selected_row = A_rows[selected_idx].take();
        // Check if A is indeed in reduced row echolon form
        debug_assert!(!selected_row[0..selected_idx].iter().any(|&val| val));
        for reduced_idx in 0..selected_idx{
            let reduced_row = A_rows[reduced_idx].take();
            if reduced_row[selected_idx]{
                (*reduced_row).iter_mut().zip(selected_row.iter()).for_each(|(reduced_val, &selected_val)| *reduced_val = (Switch(*reduced_val) - Switch(selected_val)).into());
                b[reduced_idx] = (Switch(b[reduced_idx]) - Switch(b[selected_idx])).into();
            }
            A_rows[reduced_idx].set(reduced_row);
        }
        A_rows[selected_idx].set(selected_row);
    }
    // The equation system is solved
    // If A is now the identity matrix, a unique solution was found. 
    // Otherwise one of multiple solutions was found.
    b
}