mod common;
use roudoudou::{OdooApi, OdooRpc};
use serde_json::json;

#[test]
fn reflect_stock_label_methods() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);
    let model = api.get_model("stock.label").unwrap();
    let res = model.call("get_public_methods", None, None).unwrap();
    assert_eq!(
        res,
        json!([
            "batch_compute_traceability",
            "build_traceability",
            "build_traceability_downstream",
            "build_traceability_upstram",
            "button_unassign_parent_um_label",
            "button_unassign_um_label",
            "change_attributes",
            "change_service_state",
            "close",
            "compute_concurrency_field",
            "compute_concurrency_field_with_access",
            "compute_diabeloop",
            "compute_events",
            "compute_handset_attributes",
            "compute_handset_or_manual_attributes",
            "compute_invoicing_status",
            "compute_kind",
            "compute_kit_attributes",
            "compute_pairing_attributes",
            "compute_part_attributes",
            "compute_parts",
            "compute_pump_attributes",
            "compute_transmitter_attributes",
            "create",
            "default_get",
            "do_print_label",
            "empty_um_label",
            "equal_split",
            "exists",
            "export_data",
            "get_available_qty",
            "get_metadata",
            "get_public_methods",
            "get_tags_as_string",
            "invalidate_cache",
            "make_reusable_um",
            "modified",
            "move",
            "name_create",
            "name_get",
            "name_search",
            "new",
            "onchange",
            "recompute",
            "recompute_sql",
            "reset_parts",
            "search",
            "servicing_ota_query",
            "servicing_ota_update",
            "show_traceability",
            "show_traceability_downstream",
            "show_traceability_upstream",
            "split",
            "toggle_active",
            "transfer_um",
            "unassign_um_label",
            "unlink",
            "update_simcards_invoicing_status",
            "wkf_available",
            "wkf_control",
            "wkf_draft",
            "wkf_inactive",
            "wkf_quarantine",
            "wkf_reserved",
            "wkf_unreserved",
            "write"
        ])
    );
}

#[test]
fn it_should_work_too() {
    common::setup();
    let rpc = OdooRpc::new();
    let api = OdooApi::new(rpc);
    let model = api.get_model("res.users").unwrap();
    let res = model.call("get_public_methods", None, None).unwrap();
    assert_eq!(res, json!([
        "check_can_login",
        "compute_concurrency_field",
        "compute_concurrency_field_with_access",
        "do_action_on_login",
        "exists",
        "export_data",
        "get_metadata",
        "get_public_methods",
        "invalidate_cache",
        "modified",
        "name_create",
        "name_get",
        "new",
        "onchange",
        "recompute",
        "recompute_sql",
        "refresh",
        "reset_attempt",
        "search",
        "toggle_active",
        "user_has_group_ids"
    ]));
}
