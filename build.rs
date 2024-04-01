fn main() {
	println!("cargo:rerun-if-changed=proto/wm.proto");
	println!("cargo:rerun-if-changed=proto/service.proto");
	println!("cargo:rerun-if-changed=proto/system.proto");
	println!("cargo:rerun-if-changed=proto/message.proto");

	prost_build::compile_protos(
		&[
			"proto/wm.proto",
			"proto/service.proto",
			"proto/system.proto",
			"proto/message.proto",
		],
		&["proto"],
	)
	.unwrap();
}
