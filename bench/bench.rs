extern crate bench;
extern crate rustfs;
extern crate rand;

use rustfs::{Proc, O_CREAT, O_RDWR, FileDescriptor};
use std::string::String;
use bench::{benchmark, Benchmarker};
use rand::random;
use std::iter::repeat;

static NUM: usize = 100;

macro_rules! bench {
  ($wrap:ident, $name:ident, $time:expr, |$p:ident, $filenames:ident| $task:stmt) => ({
    let $filenames = generate_names(NUM);
    let $wrap = |b: &mut Benchmarker| {
      let mut $p = Proc::new();
      b.run(|| {
        $task
      });
    };
    benchmark(stringify!($name), $wrap, $time);
  });
}

macro_rules! bench_many {
  ($wrap:ident, $name:ident, $time:expr, |$p:ident, $fd:ident, $filename:ident| $op:stmt) => ({
    let filenames = generate_names(NUM);
    let $wrap = |b: &mut Benchmarker| {
      let mut $p = Proc::new();
      b.run(|| {
        for i_j in 0..NUM {
          let $filename = &filenames[i_j];
          let $fd = $p.open($filename, O_CREAT | O_RDWR);
          $op
        }
      });
    };
    benchmark(stringify!($name), $wrap, $time);
  })
}

fn ceil_div(x: usize, y: usize) -> usize {
  return (x + y - 1) / y;
}

fn rand_array(size: usize) -> Vec<u8> {
  (0..size).map(|_| random::<u8>()).collect()
}

fn generate_names(n: usize) -> Vec<String> {
  let name_length = ceil_div(n, 26);
  let mut name: Vec<_> = repeat('@' as u8).take(name_length).collect();

  (0..n).map(|i| {
    let next = name[i / 26] + 1;
    name[i / 26] = next;

    let string_result = String::from_utf8(name.clone());
    match string_result {
      Ok(string) => string,
      Err(_) => panic!("Bad string!")
    }
  }).collect()
}

fn open_many<'a>(p: &mut Proc<'a>, names: &'a Vec<String>) -> Vec<FileDescriptor> {
  (0..names.len()).map(|i| {
    let fd = p.open(&names[i], O_CREAT | O_RDWR);
    fd
  }).collect()
}

fn close_all(p: &mut Proc, fds: &Vec<FileDescriptor>) {
  for fd in fds.iter() {
    p.close(*fd);
  }
}

fn unlink_all<'a>(p: &mut Proc<'a>, names: &'a Vec<String>) {
  for filename in names.iter() {
    p.unlink(&filename);
  }
}

#[allow(non_snake_case)]
fn main() {
  bench!(bench_OC1, OC1, 1, |p, _n| {
    let fd = p.open("test", O_CREAT);
    p.close(fd);
  });

  bench!(bench_OtC, OtC, 100, |p, filenames| {
    let fds = open_many(&mut p, &filenames);
    close_all(&mut p, &fds);
  });

  bench_many!(bench_OC, OC, 100, |p, fd, _f| {
    p.close(fd);
  });

  bench!(bench_OtCtU, OtCtU, 800, |p, filenames| {
    let fds = open_many(&mut p, &filenames);
    close_all(&mut p, &fds);
    unlink_all(&mut p, &filenames);
  });

  bench_many!(bench_OCU, OCU, 500, |p, fd, filename| {
    p.close(fd);
    p.unlink(filename);
  });

  let size = 1024;
  let content = rand_array(size);
  bench_many!(bench_OWsC, OWsC, 100, |p, fd, filename| {
    p.write(fd, &content);
    p.close(fd);
  });

  let size = 1024;
  let content = rand_array(size);
  bench_many!(bench_OWsCU, OWsCU, 100, |p, fd, filename| {
    p.write(fd, &content);
    p.close(fd);
    p.unlink(filename);
  });

  let size = 40960;
  let content = rand_array(size);
  bench_many!(bench_OWbC, OWbC, 100, |p, fd, filename| {
    p.write(fd, &content);
    p.close(fd);
  });

  let size = 40960;
  let content = rand_array(size);
  bench_many!(bench_OWbCU, OWbCU, 100, |p, fd, filename| {
    p.write(fd, &content);
    p.close(fd);
    p.unlink(filename);
  });

  let (size, many) = (1024, 4096);
  let content = rand_array(size);
  bench_many!(bench_OWMsC, OWMsC, 3000, |p, fd, filename| {
    for _ in 0..many {
      p.write(fd, &content);
    }
    p.close(fd);
  });

  let (size, many) = (1024, 4096);
  let content = rand_array(size);
  bench_many!(bench_OWMsCU, OWMsCU, 5000, |p, fd, filename| {
    for _ in 0..many {
      p.write(fd, &content);
    }
    p.close(fd);
    p.unlink(filename);
  });

  let (size, many) = (1048576, 32);
  let content = rand_array(size);
  bench_many!(bench_OWMbC, OWMbC, 5000, |p, fd, filename| {
    for _ in 0..many {
      p.write(fd, &content);
    }
    p.close(fd);
  });

  let (size, many) = (1048576, 32);
  let content = rand_array(size);
  bench_many!(bench_OWMbCU, OWMbCU, 7000, |p, fd, filename| {
    for _ in 0..many {
      p.write(fd, &content);
    }
    p.close(fd);
    p.unlink(filename);
  });

  let (start_size, many) = (2, 4096);
  let content = rand_array(start_size * many);
  bench_many!(bench_OWbbC, OWbbC, 5000, |p, fd, filename| {
    for i in 1..(many + 1) {
      p.write(fd, &content[..(i * start_size)]);
    }
    p.close(fd);
  });

  let (start_size, many) = (2, 4096);
  let content = rand_array(start_size * many);
  bench_many!(bench_OWbbCU, OWbbCU, 7000, |p, fd, filename| {
    for i in 1..(many + 1) {
      p.write(fd, &content[0..(i * start_size)]);
    }
    p.close(fd);
    p.unlink(filename);
  });
}
