struct DataIn {
    f1: f32;
    f2: f32;
};

struct DataInArray {
    data: array<DataIn>;
};

struct DataOut {
    i_start: u32;
    interval: u32;
    mean: f32;
};


struct DataOutArray {
    data: array<DataOut>;
};

[[group(0), binding(0)]]
var<storage, read_write> v_in: DataInArray;

[[group(0), binding(1)]]
var<storage, read_write> v_out: DataOutArray;

fn sum_datain_slice(i_start: u32, i_end: u32) -> f32 {
    // Sums thru [i_start, i_end)
    var s: f32 = 0.0;
    var i: u32 = i_start;
    loop {
        if (i >= i_end) {
            break;
        }
        s = s + v_in.data[i].f1;
        i = i + 1u;
    }
    return s;
}

[[stage(compute), workgroup_size(2)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let global_x_max: u32 = 50000u;
    let i = global_id.x + ((global_id.y - 1u) * global_x_max);

    v_out.data[i].mean = f32(i);
}
