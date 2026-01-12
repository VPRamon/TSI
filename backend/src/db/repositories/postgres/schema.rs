// @generated automatically by Diesel CLI.

diesel::table! {
    schedules (schedule_id) {
        schedule_id -> Int8,
        schedule_name -> Text,
        checksum -> Text,
        uploaded_at -> Timestamptz,
        dark_periods_json -> Jsonb,
        possible_periods_json -> Jsonb,
        raw_schedule_json -> Nullable<Jsonb>,
        schedule_period_json -> Jsonb,
    }
}

diesel::table! {
    schedule_blocks (scheduling_block_id) {
        scheduling_block_id -> Int8,
        schedule_id -> Int8,
        source_block_id -> Int8,
        original_block_id -> Nullable<Text>,
        priority -> Float8,
        requested_duration_sec -> Int4,
        min_observation_sec -> Int4,
        target_ra_deg -> Float8,
        target_dec_deg -> Float8,
        min_altitude_deg -> Nullable<Float8>,
        max_altitude_deg -> Nullable<Float8>,
        min_azimuth_deg -> Nullable<Float8>,
        max_azimuth_deg -> Nullable<Float8>,
        constraint_start_mjd -> Nullable<Float8>,
        constraint_stop_mjd -> Nullable<Float8>,
        visibility_periods_json -> Jsonb,
        scheduled_periods_json -> Jsonb,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    schedule_block_analytics (schedule_id, scheduling_block_id) {
        schedule_id -> Int8,
        scheduling_block_id -> Int8,
        priority_bucket -> Int2,
        requested_hours -> Float8,
        total_visibility_hours -> Float8,
        num_visibility_periods -> Int4,
        elevation_range_deg -> Nullable<Float8>,
        scheduled -> Bool,
        scheduled_start_mjd -> Nullable<Float8>,
        scheduled_stop_mjd -> Nullable<Float8>,
        validation_impossible -> Bool,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    schedule_summary_analytics (schedule_id) {
        schedule_id -> Int8,
        total_blocks -> Int4,
        scheduled_blocks -> Int4,
        unscheduled_blocks -> Int4,
        impossible_blocks -> Int4,
        scheduling_rate -> Float8,
        priority_mean -> Nullable<Float8>,
        priority_median -> Nullable<Float8>,
        priority_scheduled_mean -> Nullable<Float8>,
        priority_unscheduled_mean -> Nullable<Float8>,
        visibility_total_hours -> Float8,
        requested_mean_hours -> Nullable<Float8>,
        gap_count -> Nullable<Int4>,
        gap_mean_hours -> Nullable<Float8>,
        gap_median_hours -> Nullable<Float8>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    schedule_validation_results (validation_id) {
        validation_id -> Int8,
        schedule_id -> Int8,
        scheduling_block_id -> Int8,
        status -> Text,
        issue_type -> Nullable<Text>,
        issue_category -> Nullable<Text>,
        criticality -> Nullable<Text>,
        field_name -> Nullable<Text>,
        current_value -> Nullable<Text>,
        expected_value -> Nullable<Text>,
        description -> Nullable<Text>,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(schedule_block_analytics -> schedule_blocks (scheduling_block_id));
diesel::joinable!(schedule_block_analytics -> schedules (schedule_id));
diesel::joinable!(schedule_blocks -> schedules (schedule_id));
diesel::joinable!(schedule_summary_analytics -> schedules (schedule_id));
diesel::joinable!(schedule_validation_results -> schedule_blocks (scheduling_block_id));
diesel::joinable!(schedule_validation_results -> schedules (schedule_id));

diesel::allow_tables_to_appear_in_same_query!(
    schedule_block_analytics,
    schedule_blocks,
    schedule_summary_analytics,
    schedule_validation_results,
    schedules,
);
