struct DataIn {
    f1: f32;
    f2: f32;
};

struct DataInArray {
    data: array<DataIn>;
};

struct DataOut {
    f1: f32;
};


struct DataOutArray {
    data: array<DataOut>;
};

[[group(0), binding(0)]]
var<storage, read_write> v_in: DataInArray;

[[group(0), binding(1)]]
var<storage, read_write> v_out: DataOutArray;

[[stage(compute), workgroup_size(1)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>) {
    var s: f32 = 0.0;
    let N = 4u;

    var i: u32 = 0u;
    loop {
        if (i >= N) {
            break;
        }

        s = (s + v_in.data[i].f1) * f32(global_id.x);
        i = i + 1u;
    }

    v_out.data[global_id.x].f1 = s;

}
