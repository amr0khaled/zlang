pub struct RegisterStatus {
    name: String,
    needed: bool,
    has_value: bool,
    size: usize,
    is_inner_splits: bool,
    inner_len: usize,
    inner_splits: Option<Vec<RegisterStatus>>,
}

impl RegisterStatus {
    pub fn new(name: String, size: usize) -> Self {
        Self {
            name,
            needed: false,
            has_value: false,
            size,
            is_inner_splits: false,
            inner_splits: None,
            inner_len: 0,
        }
    }
    pub fn new_full(name: &mut String) -> Self {
        let mut split1_name = name.clone();
        split1_name.insert(0, 'e');
        let mut split2_name = name.clone();
        split2_name.pop();
        let mut split3_name = split2_name.clone();
        let mut split4_name = split2_name.clone();
        split3_name.push('h');
        split4_name.push('l');

        let split1 = RegisterStatus::new(split1_name, 32);
        let split2 = RegisterStatus::new(split2_name, 16);
        let split3 = RegisterStatus::new(split3_name, 8);
        let split4 = RegisterStatus::new(split4_name, 8);

        let splits = vec![split1, split2, split3, split4];
        let len = splits.len();

        name.insert(0, 'r');
        Self {
            name: name.clone(),
            needed: false,
            has_value: false,
            size: 64,
            is_inner_splits: true,
            inner_splits: Some(splits),
            inner_len: len,
        }
    }
    pub fn add_split(&mut self, name: String, size: usize) {
        if self.inner_splits.is_none() {
            self.inner_splits = Some(vec![]);
        }
        self.inner_splits
            .as_mut()
            .unwrap()
            .push(RegisterStatus::new(name, size));
        self.inner_len += 1;
    }
    pub fn get_split(&mut self, i: usize) -> Option<&mut RegisterStatus> {
        if i >= self.inner_len {
            panic!("That is way out of bounds");
        }
        if let Some(splits) = &mut self.inner_splits {
            let split = &mut splits[i];
            Some(split)
        } else {
            None
        }
    }
    pub fn has_value(&self) -> bool {
        self.has_value
    }
    pub fn is_needed(&self) -> bool {
        self.needed
    }
    pub fn no_value(&mut self) {
        self.has_value = false;
    }
    pub fn value(&mut self) {
        self.has_value = true;
    }
    pub fn needed(&mut self) {
        self.needed = true;
    }
    pub fn not_needed(&mut self) {
        self.needed = false;
    }
    pub fn free(&mut self) {
        self.has_value = false;
        self.needed = false;
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct CpuState {
    rax: RegisterStatus,
    rbx: RegisterStatus,
    rcx: RegisterStatus,
    rdx: RegisterStatus,
    rsi: RegisterStatus,
    rdi: RegisterStatus,
    rsp: RegisterStatus,
    rbp: RegisterStatus,
}

impl CpuState {
    pub fn new() -> Self {
        let rax = RegisterStatus::new_full(&mut String::from("ax"));
        let rbx = RegisterStatus::new_full(&mut String::from("bx"));
        let rcx = RegisterStatus::new_full(&mut String::from("cx"));
        let rdx = RegisterStatus::new_full(&mut String::from("dx"));
        let rsi = RegisterStatus::new(String::from("rsi"), 64);
        let rdi = RegisterStatus::new(String::from("rdi"), 64);
        let rsp = RegisterStatus::new(String::from("rsp"), 64);
        let rbp = RegisterStatus::new(String::from("rbp"), 64);

        Self {
            rax,
            rbx,
            rcx,
            rdx,
            rsi,
            rdi,
            rsp,
            rbp,
        }
    }
}
