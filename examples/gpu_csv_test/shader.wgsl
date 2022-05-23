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
    std_dev: f32;
    sharpe: f32;
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
    var is_all_zeros: bool = true;
    loop {
        if (i >= i_end) {
            break;
        }
        let t: f32 = v_in.data[i].f1;
        if (t!=0.0) {
            s = s + t;
            is_all_zeros = false;
        }
        i = i + 1u;
    }
    if (is_all_zeros) {
        return 0.0/0.0;
    }
    else {
        return s;
    }
}

fn std_datain_slice(i_start: u32, i_end: u32, mean: f32) -> f32 {
    // Sums thru [i_start, i_end)
    var s: f32 = 0.0;
    var i: u32 = i_start;
    var is_all_zeros: bool = true;
    loop {
        if (i >= i_end) {
            break;
        }
        let t: f32 = v_in.data[i].f1;
        if (t!=0.0) {
            var diff: f32 = t - mean;
            diff = diff*diff;
            s = s + diff;
            is_all_zeros = false;
        }
        i = i + 1u;
    }
    if (is_all_zeros) {
        return 0.0/0.0;
    }
    else {
        s = s / f32(i);
        return sqrt(s);
    }
}

[[stage(compute), workgroup_size(2)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let global_x_max: u32 = 50000u;
    let idx: u32 = global_id.x + ((global_id.y - 1u) * global_x_max);

    let i_start: u32 = v_out.data[idx].i_start;
    let interval: u32 = v_out.data[idx].interval;

    let mean: f32 = sum_datain_slice(i_start, i_start + interval) / f32(interval);
    let std_dev: f32 = std_datain_slice(i_start, i_start + interval, mean) / f32(interval);
    v_out.data[idx].mean = mean;
    v_out.data[idx].std_dev = std_dev;
    v_out.data[idx].sharpe = mean / std_dev;
}
