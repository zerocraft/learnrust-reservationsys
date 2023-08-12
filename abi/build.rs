use std::process::Command;

fn main() {
    tonic_build::configure()
        .out_dir("src/pb")
        .type_attribute("proto:package.type", "#[derive(sqlx::Type)]")
        .compile(&["protos/reservation.proto"], &["protos"])
        .unwrap();

    Command::new("cargo").args(&["fmt"]).output().unwrap();
}
