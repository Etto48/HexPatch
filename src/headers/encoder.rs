use hexpatch_keystone::Keystone;

pub enum Encoder {
    Keystone(Keystone),
    EBPF,
}
