use magnus::{function, method, prelude::*, Error, Ruby};

// #[magnus::wrap(class = "Point")]
struct Point {
    x: isize,
    y: isize,
}

// Expanded from the macro above.
impl magnus::DataTypeFunctions for Point {}
unsafe impl magnus::TypedData for Point {
    fn class(ruby: &magnus::Ruby) -> magnus::RClass {
        use magnus::{
            value::{Lazy, ReprValue},
            Class, RClass,
        };
        static CLASS: Lazy<RClass> = Lazy::new(|ruby| {
            // Sleep added here to allow others to block on the initialization.
            // This isn't required to reproduce the deadlock but adding it makes
            // the deadlock hit EVERY time.
            std::thread::sleep(std::time::Duration::from_millis(1000));
            let class: RClass = ruby
                .class_object()
                .funcall("const_get", ("Point",))
                .unwrap();
            class.undef_default_alloc_func();
            class
        });
        ruby.get_inner(&CLASS)
    }
    fn data_type() -> &'static magnus::DataType {
        static DATA_TYPE: magnus::DataType =
            ::magnus::typed_data::DataTypeBuilder::<Point>::new(unsafe {
                std::ffi::CStr::from_bytes_with_nul_unchecked("Point\u{0}".as_bytes())
            })
            .build();
        &DATA_TYPE
    }
}

impl Point {
    fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    fn x(&self) -> isize {
        self.x
    }

    fn y(&self) -> isize {
        self.y
    }

    fn distance(&self, other: &Point) -> f64 {
        (((other.x - self.x).pow(2) + (other.y - self.y).pow(2)) as f64).sqrt()
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("Point", ruby.class_object())?;
    class.define_singleton_method("new", function!(Point::new, 2))?;
    class.define_method("x", method!(Point::x, 0))?;
    class.define_method("y", method!(Point::y, 0))?;
    class.define_method("distance", method!(Point::distance, 1))?;
    Ok(())
}
