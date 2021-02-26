mod common;
// use std::hint::unreachable_unchecked;

use log::error;
use pretty_assertions::assert_eq;
// use rand::distributions::uniform::UniformDuration;
use roudoudou::{OdooClient, Method, MethodKind};
// use serde_json::json;

macro_rules! meth {
    ($n:ident, $foo:ident) => {
        Method {
            name: stringify!($n).to_owned(),
            kind: MethodKind::$foo
        }
        
    };
}
#[test]
fn reflect_stock_label_methods() {
    common::setup();
    let mut cli = OdooClient::new();
    match cli.login("ota3", "admin", "admin") {
        Err(err) => {
            error!("could not login to odoo: {}", err);
        }
        Ok(cli) => {
            let model = cli.get_model("stock.label").unwrap();

            match model.get_methods() {
                Err(_err) => {unreachable!();}
                Ok(methods) => {
                let expected = vec![
                    meth!(batch_compute_traceability, Model),
                    meth!(build_traceability, Multi),
                    meth!(build_traceability_downstream, Multi),
                    meth!(build_traceability_upstram, Multi),
                    meth!(button_unassign_parent_um_label, Multi),
                    meth!(button_unassign_um_label, Multi),
                    meth!(change_attributes, Multi),
                    meth!(change_service_state, Multi),
                    meth!(close, Multi),
                    meth!(collect_attributes, Multi),
                    meth!(collect_part_attributes, Multi),
                    meth!(collect_parts, Multi),
                    meth!(compute_cgm_variants, Multi),
                    meth!(compute_concurrency_field, One),
                    meth!(compute_concurrency_field_with_access, One),
                    meth!(compute_diabeloop, Multi),
                    meth!(compute_events, Multi),
                    meth!(compute_handset_variants, Multi),
                    meth!(compute_invoicing_status, Multi),
                    meth!(compute_kind, Multi),
                    meth!(compute_kit_variants, Multi),
                    meth!(compute_manual_variants, Multi),
                    meth!(compute_ota_update, Multi),
                    meth!(compute_pairing_variants, Multi),
                    meth!(compute_part_variants, Multi),
                    meth!(compute_parts, Multi),
                    meth!(compute_pump_variants, Multi),
                    meth!(compute_terminal_variants, Multi),
                    meth!(compute_variants, Multi),
                    meth!(create, Model),
                    meth!(default_get, Model),
                    meth!(do_print_label, Multi),
                    meth!(empty_um_label, Multi),
                    meth!(ensure_version_to_attribute_model, Model),
                    meth!(equal_split, Multi),
                    meth!(exists, Multi),
                    meth!(export_data, Multi),
                    meth!(get_available_qty, Multi),
                    meth!(get_metadata, Multi),
                    meth!(get_public_methods, Model),
                    meth!(get_tags_as_string, One),
                    meth!(invalidate_cache, Model),
                    meth!(make_reusable_um, Multi),
                    meth!(modified, Multi),
                    meth!(move, Multi),
                    meth!(name_create, Model),
                    meth!(name_get, Multi),
                    meth!(name_search, Model),
                    meth!(new, Model),
                    meth!(onchange, Multi),
                    meth!(recompute, Model),
                    meth!(recompute_sql, Model),
                    meth!(reset_parts, Multi),
                    meth!(search, Model),
                    meth!(servicing_ota_historize, Multi),
                    meth!(servicing_ota_init_history, Multi),
                    meth!(servicing_ota_query, Multi),
                    meth!(servicing_ota_update, Multi),
                    meth!(show_traceability, Multi),
                    meth!(show_traceability_downstream, Multi),
                    meth!(show_traceability_upstream, Multi),
                    meth!(split, Multi),
                    meth!(toggle_active, Multi),
                    meth!(transfer_um, Multi),
                    meth!(unassign_um_label, Multi),
                    meth!(unlink, Multi),
                    meth!(update_simcards_invoicing_status, Model),
                    meth!(wkf_available, One),
                    meth!(wkf_control, Multi),
                    meth!(wkf_draft, Multi),
                    meth!(wkf_inactive, Multi),
                    meth!(wkf_quarantine, Multi),
                    meth!(wkf_reserved, Multi),
                    meth!(wkf_unreserved, Multi),
                    meth!(write, Multi)
                   ];
                   assert_eq!(methods, expected)
                }
            }
            // cli.logout().unwrap();
        }
    }
}
    


#[test]
fn  reflect_res_users_methods() {
    common::setup();
    let mut cli = OdooClient::new();
    match cli.login("ota3", "admin", "admin") {
        Err(err) => {
            error!("could not login to odoo: {}", err);
        }
        Ok(cli) => {
            let model = cli.get_model("res.users").unwrap();
            match model.get_methods() {
                Err(err) => {error!("could not get methods for res.users: {}", err)}
                Ok(methods) => {
                    let expected = vec![
                        // meth!(check_can_login, Multi),
                        meth!(compute_concurrency_field, One),
                        meth!(compute_concurrency_field_with_access, One),
                        // meth!(do_action_on_login, Multi),
                        meth!(exists, Multi),
                        meth!(export_data, Multi),
                        meth!(get_metadata, Multi),
                        meth!(get_public_methods, Model),
                        meth!(invalidate_cache, Model),
                        meth!(modified, Multi),
                        meth!(name_create, Model),
                        meth!(name_get, Multi),
                        meth!(new, Model),
                        meth!(onchange, Multi),
                        meth!(recompute, Model),
                        meth!(recompute_sql, Model),
                        meth!(refresh, Model),
                        // meth!(reset_attempt, Multi),
                        // meth!(search, Multi),
                        meth!(toggle_active, Multi),
                        meth!(user_has_group_ids, Multi)
                        
                    ];
                    assert_eq!(methods, expected);

                }
            }
            // cli.logout().unwrap();
        }
    }
} 
