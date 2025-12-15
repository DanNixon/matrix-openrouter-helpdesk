use metrics::counter;

pub fn record_request_metric(user_id: &str, room_id: &str, result: &str) {
    counter!(
        "helpdesk_requests_total",
        "matrix_user" => user_id.to_string(),
        "matrix_room" => room_id.to_string(),
        "result" => result.to_string()
    )
    .increment(1);
}
