struct DataIn {
    value: f32;
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

[[stage(compute), workgroup_size(2)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    let global_x_max: u32 = 50000u;
    let idx: u32 = global_id.x + ((global_id.y - 1u) * global_x_max);

    //let storage_buffer_length = 8324640u;
    //let minutes_in_a_day = 1440u;
    let days_in_buffer = 5781u;
    let i_start: u32 = v_out.data[idx].i_start;
    let interval: u32 = v_out.data[idx].interval;

    var returns: array<f32, 5781u>;

    // Collect returns, mean of returns
    var i = 0u;
    var mean_summand: f32 = 0.0;
    loop {
        if (i >= days_in_buffer){
            break;
        }
        let start = days_in_buffer * i + i_start;
        let end = start + interval;

        let ret = (v_in.data[end].value - v_in.data[start].value)*100.0;
        returns[i] = ret;

        mean_summand = mean_summand + ret;

        i = i + 1u;
    }
    let mean = mean_summand / f32(i);

    // Calc std dev
    var j = 0u;
    var std_summand: f32 = 0.0;
    loop {
        if (j >= i) {
            break;
        }
        let s = returns[j] - mean;
        std_summand = std_summand + (mean*mean);

        j = j + 1u;
    }
    let std_dev = sqrt(std_summand/f32(j));

    v_out.data[idx].mean = mean;
    v_out.data[idx].std_dev = std_dev;
    v_out.data[idx].sharpe = mean / std_dev;
}
