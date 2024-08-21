pub fn pack_to_vec<T>(val: T) -> Vec<u8>
where
    T: solana_sdk::program_pack::Pack,
{
    let mut buf = vec![0; T::LEN];
    T::pack(val, &mut buf).unwrap();

    buf
}
