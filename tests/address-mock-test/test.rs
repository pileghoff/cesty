use cesty::mem_mock::Memmock;
use proptest::prelude::*;

unsafe extern "C" {
    pub fn store_int(val: u32, dst: *const u32);
    pub fn load_int(src: *const u32) -> u32;

    pub fn store_byte(val: u8, dst: *const u8, offset: std::ffi::c_int);
    pub fn load_byte(src: *const u8, offset: std::ffi::c_int) -> u8;
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_basic_read(ptr: usize, val: u32) {
        let mem_mock = Memmock::new();
        mem_mock.set(ptr, val.to_ne_bytes().into_iter().collect());

        assert_eq!(unsafe { load_int(ptr as *const u32) }, val);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_basic_write(ptr: usize, val: u32) {
        let mem_mock = Memmock::new();
        mem_mock.set(ptr, (0u32).to_ne_bytes().into_iter().collect());

        unsafe { store_int(val, ptr as *const u32) };

        assert_eq!(mem_mock.get(ptr).unwrap(), val.to_ne_bytes());
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_multiple_read_writes(pairs: Vec<(usize, u32)>) {
        let mem_mock = Memmock::new();
        for (a, _) in pairs.iter() {
            mem_mock.set(*a, (0u32).to_ne_bytes().into_iter().collect());
        }

        for (a, v) in pairs.iter() {
            unsafe { store_int(*v, *a as *const u32) };
            assert_eq!(unsafe { load_int(*a as *const u32) }, *v);
        }


    }
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_multiple_read_writes_same_addr(mut pairs: Vec<(usize, Vec<u32>)>) {
        let mem_mock = Memmock::new();
        let mut index = 0;
        while !pairs.is_empty() {
            if index >= pairs.len() {
                index = 0;
            }

            let addr = pairs[index].0;
            if let Some(value) = pairs[index].1.pop() {
                if mem_mock.get(addr).is_none() {
                    mem_mock.set(addr,  (0u32).to_ne_bytes().into_iter().collect());
                }

            unsafe { store_int(value, addr as *const u32) };
            assert_eq!(unsafe { load_int(addr as *const u32) }, value);


            } else {
                pairs.remove(index);
            }


            index += 1;
        }


    }
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_basic_read_byte(ptr: usize, val: u32) {
        let mem_mock = Memmock::new();
        mem_mock.set(ptr, val.to_ne_bytes().into_iter().collect());

        assert_eq!(unsafe {
            load_byte(ptr as *const u8, 0) }, (val & 0xff) as u8);

        assert_eq!(unsafe {
            load_byte(ptr as *const u8, 1) }, ((val >> 8) & 0xff) as u8);

        assert_eq!(unsafe {
            load_byte(ptr as *const u8, 2) }, ((val >> 16) & 0xff) as u8);

        assert_eq!(unsafe {
            load_byte(ptr as *const u8, 3) }, ((val >> 24) & 0xff) as u8);
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
            fork: true,
            .. ProptestConfig::default()
        })]
    #[test]
    fn test_basic_write_byte(ptr: usize, val: [u8;4]) {
        let mem_mock = Memmock::new();
        mem_mock.set(ptr, (0u32).to_ne_bytes().into_iter().collect());

        unsafe { store_byte(val[0], ptr as *const u8, 0) };
        unsafe { store_byte(val[1], ptr as *const u8, 1) };
        unsafe { store_byte(val[2], ptr as *const u8, 2) };
        unsafe { store_byte(val[3], ptr as *const u8, 3) };

        assert_eq!(mem_mock.get(ptr).unwrap(), val);
        assert_eq!(mem_mock.get(ptr+1).unwrap(), val[1..]);
        assert_eq!(mem_mock.get(ptr+2).unwrap(), val[2..]);
        assert_eq!(mem_mock.get(ptr+3).unwrap(), val[3..]);
    }
}
