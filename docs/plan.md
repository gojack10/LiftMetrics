# Project Plan Checklist

Okay, this is a fantastic starting point for a really useful personal app! As a professional Rust programmer and brainstormer, let's dive deep and flesh this out.

**I. Core Philosophy & Design Considerations**

**Simplicity and Focus:** Maintain straightforward flow, keep code as simple as possible.

**II. Database Schema (SQLite)**

Refine the schema using ISO8601 strings for dates/datetimes.

- [x] **`diet_cycles`**
    - [x] `id` INTEGER PRIMARY KEY AUTOINCREMENT
    - [x] `phase` TEXT NOT NULL
    - [x] `start_date` TEXT NOT NULL
    - [x] `planned_end_date` TEXT NOT NULL
    - [x] `actual_end_date` TEXT (NULLABLE)
    - [x] `notes` TEXT
    - [x] `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP

- [x] **`weight_logs`**
    - [x] `id` INTEGER PRIMARY KEY AUTOINCREMENT
    - [x] `diet_cycle_id` INTEGER (FK to `diet_cycles.id`, NULLABLE)
    - [x] `log_date` TEXT NOT NULL
    - [x] `weight_lbs` REAL NOT NULL
    - [x] `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP

- [x] **`exercises`**
    - [x] `id` INTEGER PRIMARY KEY AUTOINCREMENT
    - [x] `name` TEXT NOT NULL UNIQUE COLLATE NOCASE
    - [x] `description` TEXT
    - [x] `default_metric_to_track` TEXT
    - [x] `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP

- [x] **`workout_sessions`**
    - [x] `id` INTEGER PRIMARY KEY AUTOINCREMENT
    - [x] `session_date` TEXT NOT NULL
    - [x] `notes` TEXT
    - [x] `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP

- [x] **`exercise_sets`**
    - [x] `id` INTEGER PRIMARY KEY AUTOINCREMENT
    - [x] `workout_session_id` INTEGER NOT NULL (FK to `workout_sessions.id`)
    - [x] `exercise_id` INTEGER NOT NULL (FK to `exercises.id`)
    - [x] `set_order` INTEGER NOT NULL
    - [x] `reps` INTEGER NOT NULL
    - [x] `weight_lbs` REAL NOT NULL
    - [x] `rpe` REAL (Optional)
    - [x] `notes` TEXT (Optional)
    - [x] `created_at` TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP

**III. Egui App State and Structure (`MyApp`)**

- [x] Define `MyApp` struct with necessary state variables:
    - [x] `active_tab`
    - [x] `db_conn` (Arc<Mutex<Connection>>)
    - [x] `log_weight_input_lbs`
    - [x] `show_diet_cycle_popup`
    - [x] `new_diet_phase`
    - [x] `new_diet_start_date`
    - [x] `new_diet_planned_end_date`
    - [x] `active_diet_cycle_id`
    - [x] `log_exercise_date`
    - [x] `current_exercises_log` (Vec<ExerciseLogEntry>)
    - [x] `exercise_name_input_buffer`
    - [x] `available_exercise_names` (Vec<String>)
    - [x] `weight_progress_data` (Vec<(f64, f64)>)
    - [x] `smoothed_weight_progress_data` (Vec<(f64, f64)>)
    - [x] `weight_plot_memory`
    - [x] `exercise_progress_selected_exercise_id`
    - [x] `all_exercises_for_dropdown` (Vec<(i64, String)>)
    - [x] `exercise_progress_data` (Vec<(f64, f64)>)
    - [x] `smoothed_exercise_progress_data` (Vec<(f64, f64)>)
    - [x] `exercise_plot_memory`
    - [x] `selected_exercise_metric`
    - [x] `status_message`
    - [x] `last_status_time`
- [x] Define helper structs: `ExerciseLogEntry`, `SetEntry`.
- [x] Define enums: `Tab`, `DietPhase`, `ExerciseMetric`.

**IV. Detailed Tab Flows & Brainstorming Extensions**

**A. Global Considerations / On Startup:**

- [x] 1. **DB Initialization:** Create SQLite file on first run, execute `CREATE TABLE IF NOT EXISTS ...` statements.
- [x] 2. **Load Initial State:** Check for active `diet_cycle`, pre-load exercise names for autocomplete and dropdown, set system date for date pickers.

**B. Log Weight Tab:**

- [ ] 1. **Diet Cycle Setup (Modal/Popup):** Implement UI for phase, start date, planned end date input. Add validation and DB insertion. Include "Skip for now" option.
- [ ] 2. **Weight Input:** Implement text field for weight input, parse to `f64`, validate, and add "Log Weight" button.
- [ ] 3. **On Log:** Insert into `weight_logs`, show confirmation
- [ ] 4. ability to edit weight logs, delete weight logs

**C. Log Exercise Tab:**

- [ ] 1. **Date Picker:** Implement date picker defaulting to today, allowing selection of past dates.
- [ ] 2. **Dynamic Exercise List (`current_exercises_log`):** Implement UI for adding/removing exercises.
- [ ] 3. **For each `ExerciseLogEntry`:**
    - [ ] **Exercise Name:** Implement text input with autocomplete from `available_exercise_names`. Add new exercises to DB and update `available_exercise_names`.
    - [ ] **Sets and Reps:** Implement dynamic list of `SetEntry` with inputs for reps, weight, rpe. Add "Add Set", "Copy previous set", and "Remove Set" buttons.
- [ ] 4. **"Log Workout" Button:** Implement button with validations and DB operations (insert into `workout_sessions` and `exercise_sets` in a transaction). Show confirmation and clear list.

**D. Weight Progress Tab:**

- [ ] 1. **Data Fetching:** Query `weight_logs`, filter by diet cycle (or show all), convert dates to timestamps.
- [ ] 2. **Smoothing:** Apply a smoothing algorithm (e.g., 7-day SMA or EMA).
- [ ] 3. **Plotting (`egui_plot`):** Plot raw and smoothed data with time on X-axis and weight on Y-axis. Add legend, remember plot memory, and reset view on 'R' key press. Show instructions.
- [ ] 4. **Diet Cycle End & Save Chart:** Implement saving chart data (CSV or image) when a diet cycle ends.

**E. Exercise Progress Tab:**

- [ ] 1. **Exercise Selection:** Implement dropdown populated with `all_exercises_for_dropdown`.
- [ ] 2. **Metric Selection:** Implement dropdown/radio buttons for `ExerciseMetric`, defaulting to `exercises.default_metric_to_track`.
- [ ] 3. **Data Fetching & Calculation:** Query `exercise_sets` and `workout_sessions`, filter by exercise, group by date, calculate chosen metric for each session.
- [ ] 4. **Smoothing & Plotting:** Apply smoothing and plot raw (optional) and smoothed data using `egui_plot`. Use exercise plot memory and 'R' key reset.

**V. Further Brainstorming & Enhancements (Beyond Initial Scope)**

- [ ] **Workout Templates:** Allow users to define and load workout templates.
- [ ] **PR Tracking:** Automatically detect and highlight Personal Records. Add a "PRs" tab.
- [ ] **Body Measurements:** Add `body_measurements` table and a tab for logging and graphing.
- [ ] **Calorie/Macro Tracking Integration:** Allow logging daily calories/macros and correlating with weight changes.
- [ ] **Import/Export:** Implement full DB backup/restore and CSV import/export.
- [ ] **Settings/Preferences:** Add options for units, theme, date format.
- [ ] **Dashboard Tab:** Create a summary view with current weight, mini-graph, upcoming diet end, last workout summary.
- [ ] **Notes & Journaling:** Add more extensive notes fields or a dedicated journal entry per day.
- [ ] **Exercise Categorization/Filtering:** Add `category` to `exercises` table and allow filtering.
- [ ] **Estimated 1RM Formulas:** Allow user to choose from different e1RM formulas in settings.
- [ ] **UI for Managing Diet Cycles:** Add a view to list, edit, and manage diet cycles.
- [ ] **Rest Timer:** Implement a simple timer widget.

**VI. Rust Crates to Consider:**

- [ ] `eframe` / `egui`: For the GUI.
- [ ] `egui_plot`: For plotting.
- [ ] `rusqlite`: For SQLite interaction.
- [x] `chrono`: For all date/time handling.
- [ ] `directories-rs`: To find user-specific data directories.
- [ ] `log`: For application logging.
- [ ] `serde` (with `serde_json` or `serde_rusqlite`): For serialization/deserialization.
- [x] Possibly `egui_extras` for `DatePickerButton` or `Table`.
- [ ] `image`: If saving plots to image.
- [ ] `plotters` (and `plotters-bitmap` / `plotters-svg`): If generating standalone image files for charts.

**VII. Implementation Strategy:**

- [x] 1. **Setup Basic Egui App:** Get a window up with the 4 tabs.
- [x] 2. **DB Layer:** Implement `db.rs` with connection setup and functions to create tables.
- [ ] 3. **Log Weight:** Implement UI and DB interaction for diet cycle setup and weight logging.
- [ ] 4. **Log Exercise:** Design dynamic list UI, implement DB interaction, add autocomplete.
- [ ] 5. **Weight Progress:** Fetch data, implement basic plot, add smoothing, add zoom/pan/reset.
- [ ] 6. **Exercise Progress:** Implement exercise selection, metric calculation, plotting.
- [ ] 7. **Refinements:** Add status messages, input validation, error handling, chart saving.
- [ ] 8. **Branch Out:** Start picking enhancements from section V.
