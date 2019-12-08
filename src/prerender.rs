pub(crate) struct PrerenderWriter {
    buf: Vec<u8>,
}

impl PrerenderWriter {
    pub(crate) fn new() -> Self {
        Self {
            buf: vec![],
        }
    }
    pub(crate) fn append(&mut self, mut data: Vec<u8>) {
        let buf = &mut self.buf;
        let mut len = data.len();
        while len > 0 {
            let low = len % 256;
            buf.push(low as u8);
            len = len >> 8;
        }
        buf.push(0);
        buf.append(&mut data);
    }
    pub(crate) fn end(self) -> Vec<u8> {
        self.buf
    }
}

pub struct PrerenderReader {
    buf: Box<[u8]>,
    pos: usize,
}

impl PrerenderReader {
    pub(crate) fn new(buf: Box<[u8]>) -> Self {
        Self {
            buf,
            pos: 0,
        }
    }
    pub(crate) fn next<'b>(&'b mut self) -> &'b [u8] {
        let buf = &self.buf;
        let mut pos = self.pos;
        let mut len = 0usize;
        let mut ml = 0;
        loop {
            let b = buf[pos] as usize;
            pos += 1;
            if b == 0 {
                break;
            }
            len += b << ml;
            ml += 8;
        }
        let ret = &buf[pos..(pos + len)];
        self.pos += pos + len;
        ret
    }
}

