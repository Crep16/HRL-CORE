use super::*;

#[test]
fn local_event_times_use_the_workshop_timezone_and_preserve_explicit_offsets() {
    for (local, zone, expected) in [
        (
            "2026-11-19T17:30",
            "America/Cancun",
            "2026-11-19T22:30:00.000Z",
        ),
        (
            "2026-11-19T23:30:15.123",
            "America/Cancun",
            "2026-11-20T04:30:15.123Z",
        ),
        (
            "2026-11-19T17:30",
            "America/Mexico_City",
            "2026-11-19T23:30:00.000Z",
        ),
        (
            "2026-07-19T17:30",
            "America/Tijuana",
            "2026-07-20T00:30:00.000Z",
        ),
        (
            "2026-11-19T17:30",
            "America/Tijuana",
            "2026-11-20T01:30:00.000Z",
        ),
        (
            "2026-11-19T17:30",
            "America/Hermosillo",
            "2026-11-20T00:30:00.000Z",
        ),
        ("2026-11-19T17:30", "UTC", "2026-11-19T17:30:00.000Z"),
        (
            "2026-11-19T17:30:00-05:00",
            "America/Tijuana",
            "2026-11-19T22:30:00.000Z",
        ),
        (
            "2026-11-19T17:30:00Z",
            "America/Cancun",
            "2026-11-19T17:30:00.000Z",
        ),
        ("2026-11-19", "America/Cancun", "2026-11-19T00:00:00.000Z"),
    ] {
        assert_eq!(
            PublicTimestamp::from_workshop_time(local, zone).unwrap(),
            PublicTimestamp::parse(expected).unwrap()
        );
    }
}

#[test]
fn local_event_times_reject_unknown_zones_invalid_dates_and_dst_ambiguity() {
    for (value, zone) in [
        ("2026-11-19T17:30", "Invalid/Zone"),
        ("2026-02-30T17:30", "America/Cancun"),
        ("2026-11-19T25:30", "America/Cancun"),
        ("2026-11-19T17:30:00.123456", "America/Cancun"),
        ("2026-03-08T02:30", "America/Tijuana"),
        ("2026-11-01T01:30", "America/Tijuana"),
    ] {
        assert!(
            PublicTimestamp::from_workshop_time(value, zone).is_err(),
            "{value} {zone}"
        );
    }
}
use crate::domain::repair_contracts::{Event, Warranty};
use serde_json::{Value, json};

fn input() -> ProjectionInput {
    ProjectionInput {
        equipment: PublicEquipment::new("Laptop".into(), "HP".into(), "Pavilion 15".into())
            .unwrap(),
        technical_status: "En diagnóstico".into(),
        stages: vec![PublicStage::new("Diagnóstico".into(), false, None).unwrap()],
        events: vec![],
        public_note: String::new(),
        warranties: vec![],
        payment: Some(PublicPayment::new("MXN".into(), 0, 0, 0).unwrap()),
        sections: PublicSections {
            events: false,
            note: false,
            warranties: false,
            payment: true,
        },
        received_at: Some(PublicTimestamp::parse("2026-11-19T17:30:00.000Z").unwrap()),
        estimated_delivery_at: None,
        delivered_at: None,
    }
}

fn document(input: ProjectionInput) -> Value {
    serde_json::to_value(
        PublicProjection::publish("ABCD-EFGH-JKMN".into(), 1, input)
            .unwrap()
            .sign()
            .unwrap(),
    )
    .unwrap()
}

fn event(public: bool) -> Event {
    Event {
        id: 42,
        date: "2026-11-19T12:30:00-05:00".into(),
        title: "Avance público".into(),
        detail: "Diagnóstico terminado".into(),
        stage: "Referencia interna".into(),
        public,
    }
}

#[test]
fn fingerprint_matches_the_portal_javascript_contract_vector() {
    let signed = PublicProjection::publish("ABCD-EFGH-JKMN".into(), 1, input())
        .unwrap()
        .sign()
        .unwrap();
    assert_eq!(
        signed.fingerprint(),
        "67a5a5d3280cacb5f2f5300869ac8dd41804508a82d3784b1a05f3f851646dab"
    );
    assert_eq!(signed.public_id(), "ABCD-EFGH-JKMN");
    assert_eq!(signed.revision(), 1);
    let value = serde_json::to_value(signed).unwrap();
    assert!(value.get("actualizadoEn").is_none());
    assert_eq!(value["versionContrato"], 1);
}

#[test]
fn only_public_event_fields_cross_the_boundary() {
    let mut source = input();
    let mut private = event(false);
    private.title = "Contraseña privada 1234".into();
    private.date = "fecha privada inválida".into();
    source.events = vec![event(true), private];
    source.sections.events = true;
    let value = document(source);
    assert_eq!(
        value["eventos"],
        json!([{
            "fecha":"2026-11-19T17:30:00.000Z", "titulo":"Avance público", "detalle":"Diagnóstico terminado"
        }])
    );
    assert_eq!(
        value["equipo"],
        json!({"tipo":"Laptop", "marca":"HP", "modelo":"Pavilion 15"})
    );
    for field in [
        "folio",
        "cliente",
        "telefono",
        "serie",
        "tecnico",
        "notaInterna",
        "costos",
        "auditoria",
    ] {
        assert!(value.get(field).is_none(), "{field}");
    }
    assert!(!value.to_string().contains("1234"));
    assert!(!value.to_string().contains("Referencia interna"));
}

#[test]
fn hidden_sections_are_absent_and_visible_empty_lists_are_preserved() {
    let mut hidden = input();
    hidden.sections.payment = false;
    hidden.public_note = "x".repeat(5000);
    hidden.events = vec![Event {
        date: "inválida".into(),
        ..event(true)
    }];
    hidden.warranties = vec![Warranty {
        name: "".into(),
        days: -1,
    }];
    let hidden = document(hidden);
    for field in ["eventos", "notaPublica", "garantias", "pago"] {
        assert!(hidden.get(field).is_none());
    }
    let mut visible = input();
    visible.sections.events = true;
    visible.sections.note = true;
    visible.sections.warranties = true;
    let visible = document(visible);
    assert_eq!(visible["eventos"], json!([]));
    assert_eq!(visible["garantias"], json!([]));
    assert_eq!(visible["notaPublica"], "");
}

#[test]
fn payments_preserve_credit_and_validate_the_view_balance() {
    let mut source = input();
    source.payment = Some(PublicPayment::new("MXN".into(), 60_000, 75_000, 0).unwrap());
    assert_eq!(
        document(source)["pago"],
        json!({"moneda":"MXN", "totalCentavos":60000,
        "cobradoCentavos":75000, "pendienteCentavos":0})
    );
    assert!(PublicPayment::new("MXN".into(), 60_000, 30_000, 30_000).is_ok());
    assert!(PublicPayment::new("MXN".into(), 60_000, 75_000, 15_000).is_err());
    assert!(PublicPayment::new("MXN".into(), 60_000, 30_000, 0).is_err());
    assert!(PublicPayment::new("mxn".into(), 60_000, 0, 60_000).is_err());
    assert!(PublicPayment::new("MXN".into(), 0, -1, 0).is_err());
    assert!(
        PublicPayment::new("MXN".into(), MAX_SAFE_INTEGER + 1, 0, MAX_SAFE_INTEGER + 1).is_err()
    );
    let mut missing = input();
    missing.payment = None;
    assert!(PublicProjection::publish("PUBLICO".into(), 1, missing).is_err());
}

#[test]
fn delivery_alone_starts_exactly_seven_days_of_access() {
    let mut source = input();
    source.technical_status = "Listo para entrega".into();
    source.stages = vec![
        PublicStage::new(
            "Reparación terminada".into(),
            true,
            Some(PublicTimestamp::parse("2026-11-19T17:30:00Z").unwrap()),
        )
        .unwrap(),
    ];
    let undelivered = document(source);
    assert!(undelivered.get("entregadoEn").is_none());
    assert!(undelivered.get("accesoHasta").is_none());
    let mut source = input();
    source.delivered_at = Some(PublicTimestamp::parse("2026-12-29T23:59:59.500-05:00").unwrap());
    let delivered = document(source);
    assert_eq!(delivered["entregadoEn"], "2026-12-30T04:59:59.500Z");
    assert_eq!(delivered["accesoHasta"], "2027-01-06T04:59:59.500Z");
}

#[test]
fn withdrawal_has_no_tracking_content_and_keeps_expiration() {
    let delivery = PublicTimestamp::parse("2026-11-19T17:30:00Z").unwrap();
    let value = serde_json::to_value(
        PublicProjection::withdraw("PUBLICO".into(), 2, Some(delivery))
            .unwrap()
            .sign()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["publicado"], false);
    assert_eq!(value["accesoHasta"], "2026-11-26T17:30:00.000Z");
    assert_eq!(value.as_object().unwrap().len(), 7);
    assert!(value.get("equipo").is_none());
    assert!(value.get("pago").is_none());
}

#[test]
fn custom_stages_keep_their_order_and_dates_require_actual_instants() {
    let mut source = input();
    source.stages = vec![
        PublicStage::new("Microsoldadura propia".into(), true, None).unwrap(),
        PublicStage::new("Prueba extendida".into(), false, None).unwrap(),
    ];
    let value = document(source);
    assert_eq!(value["etapas"][0]["nombre"], "Microsoldadura propia");
    assert_eq!(value["etapas"][0]["completada"], true);
    assert_eq!(value["etapas"][1]["nombre"], "Prueba extendida");
    assert!(PublicTimestamp::parse("2026-11-19").is_err());
    assert!(PublicTimestamp::parse("2026-11-19T12:30:00").is_err());
    assert!(PublicTimestamp::parse("2026-02-30T12:30:00Z").is_err());
    assert!(PublicTimestamp::parse("2026-11-19T12:30:00.000001Z").is_err());
    let at = PublicTimestamp::parse("2026-11-19T12:30:00Z").unwrap();
    assert!(PublicStage::new("Pendiente".into(), false, Some(at)).is_err());
}

#[test]
fn invalid_public_data_is_rejected_without_truncating_the_local_source() {
    for count in [31, 201, 21] {
        let mut source = input();
        match count {
            31 => {
                source.stages =
                    vec![PublicStage::new("Personalizada".into(), false, None).unwrap(); count]
            }
            201 => {
                source.sections.events = true;
                source.events = vec![event(true); count];
            }
            _ => {
                source.sections.warranties = true;
                source.warranties = vec![
                    Warranty {
                        name: "Garantía".into(),
                        days: 30
                    };
                    count
                ];
            }
        }
        assert!(PublicProjection::publish("PUBLICO".into(), 1, source).is_err());
    }
    assert!(PublicEquipment::new(" ".into(), "".into(), "".into()).is_err());
    assert!(PublicEquipment::new("Laptop".into(), "x".repeat(61), "".into()).is_err());
    assert!(PublicProjection::withdraw("folio/con/ruta".into(), 1, None).is_err());
    assert!(PublicProjection::withdraw("PUBLICO".into(), 0, None).is_err());
    assert!(PublicProjection::withdraw("PUBLICO".into(), MAX_SAFE_INTEGER + 1, None).is_err());
}

#[test]
fn canonical_keys_follow_javascript_utf16_and_unicode_is_not_escaped() {
    let source = json!({"\u{e000}":"á", "\u{10000}": ["uno", "dos"], "a": {"z": 2, "b": 1}});
    let mut canonical = String::new();
    canonical_json(&source, &mut canonical).unwrap();
    assert_eq!(
        canonical,
        "{\"a\":{\"b\":1,\"z\":2},\"𐀀\":[\"uno\",\"dos\"],\"\u{e000}\":\"á\"}"
    );
}
