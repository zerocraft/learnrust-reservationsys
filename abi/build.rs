use std::process::Command;

use tonic_build::Builder;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        //.type_attribute("proto:package.type", "#[derive(sqlx::Type)]")
        .type_attribute(
            "reservation.Reservation",
            "#[derive(derive_builder::Builder)]",
        )
        .field_attribute("reservation.Reservation.id", "#[builder(setter(skip))]")
        .with_builder_attribute(
            "reservation.Reservation",
            &["uid", "resource_id", "note", "start", "end", "rstatus"],
            "#[builder(default)]",
        )
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(["fmt"]).output().unwrap();
}

trait BuilderExt {
    fn with_builder_attribute(self, path: &str, fields: &[&str], attr: &str) -> Self;
}

impl BuilderExt for Builder {
    fn with_builder_attribute(self, path: &str, fields: &[&str], attr: &str) -> Self {
        fields.iter().fold(self, |acc, f| {
            acc.field_attribute(format!("{}.{}", path, f), attr)
        })
    }
}
