use super::*;

#[test]
fn flatten_job_id_one_array() {
    let input = vec![
        SacctOutput {
            job_id: "43214_432".to_string(),
            job_name: "test name".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_[0-4]".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "45344_432645".to_string(),
            job_name: "tesme".to_string(),
            state: "same state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "43645_42".to_string(),
            job_name: "tme".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
    ];

    let expected = vec![
        SacctOutput {
            job_id: "43214_432".to_string(),
            job_name: "test name".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_0".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_1".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_2".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_3".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_4".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "45344_432645".to_string(),
            job_name: "tesme".to_string(),
            state: "same state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "43645_42".to_string(),
            job_name: "tme".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
    ];

    let output = flatten_job_id(input);
    assert_eq!(expected, output);
}

#[test]
fn flatten_job_id_no_flat() {
    let input = vec![
        SacctOutput {
            job_id: "43214_432".to_string(),
            job_name: "test name".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "432243_2".to_string(),
            job_name: "tese".to_string(),
            state: "samplstate".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "45344_432645".to_string(),
            job_name: "tesme".to_string(),
            state: "same state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
        SacctOutput {
            job_id: "43645_42".to_string(),
            job_name: "tme".to_string(),
            state: "sample state".to_string(),
            slurm_exit_code: 0,
            program_exit_code: 0,
        },
    ];

    let output = flatten_job_id(input.clone());
    assert_eq!(input, output);
}
