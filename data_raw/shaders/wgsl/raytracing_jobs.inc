
fn extract_job_index(atomic_index: u32) -> i32 {
    var current = atomicLoad(&jobs[atomic_index]); 
    if (current == 0u) {
        return -1;
    }
    let index = countTrailingZeros(current & (current - 1u)) - 1u;
    while (current > 0u) {
        let v = current & ~(1u << index);
        let result = atomicCompareExchangeWeak(&jobs[atomic_index], current, v);
        if (result.exchanged) {
            break;
        }
        current = result.old_value;
    }
    if (current > 0u) {
        return i32(index);
    }
    return -1;
}

fn insert_job_index(atomic_index: u32) -> i32 {
    let full = i32(0xFFFFFFFFu);
    var current = i32(atomicLoad(&jobs[atomic_index])); 
    if (current == full) {
        return -1;
    }
    let index = i32(countTrailingZeros(u32(current & (current - 1))) - 1u);
    while (current != full) {
        let v = u32(current | (1 << u32(index)));
        let result = atomicCompareExchangeWeak(&jobs[atomic_index], u32(current), v);
        if (result.exchanged) {
            break;
        }
        current = i32(result.old_value);
    }
    if (current == full) {
        return index;
    }
    return -1;
}

fn fill_job(job_index: u32) {
    let atomic_index = job_index / 32u;
    let i = job_index % 32u;
    atomicOr(&jobs[atomic_index], 1u << i);
}

fn find_empty_atomic(max_jobs_count: u32) -> i32 {
    var atomic_index = 0u;
    var result = -1;
    while (atomic_index < max_jobs_count && result < 0) {
        let current = atomicLoad(&jobs[atomic_index]); 
        if (current >= 0u && current < 0xFFFFFFFFu) {
            result = i32(atomic_index);
            break;
        }
        atomic_index = atomic_index + 1u;
    }
    return result;
}

fn find_busy_atomic(current_atomic: u32, max_jobs_count: u32) -> i32 {
    let available = atomicLoad(&jobs[current_atomic]); 
    if(available > 0u) {
        return i32(current_atomic);
    }
    var atomic_index = current_atomic + 1u;
    while (atomic_index != current_atomic) {
        let current = atomicLoad(&jobs[atomic_index]); 
        if (current > 0u) {
            return i32(atomic_index);
        }
        atomic_index = (atomic_index + 1u) % max_jobs_count;
    }
    return -1;
}