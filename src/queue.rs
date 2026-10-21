use crate::model::{DownloadOptions, DownloadStatus, DownloadTask};

#[derive(Default)]
pub struct DownloadQueue {
    tasks: Vec<DownloadTask>,
}

impl DownloadQueue {
    pub fn add(&mut self, options: DownloadOptions) -> String {
        let id = format!("download-{}", self.tasks.len() + 1);
        self.tasks.push(DownloadTask::new(&id, options));
        id
    }

    pub fn tasks(&self) -> &[DownloadTask] {
        &self.tasks
    }

    pub fn tasks_mut(&mut self) -> &mut [DownloadTask] {
        &mut self.tasks
    }

    pub fn retry_failed(&mut self) {
        for task in &mut self.tasks {
            if task.status == DownloadStatus::Failed {
                task.status = DownloadStatus::Queued;
                task.error = None;
            }
        }
    }

    pub fn next_queued(&self) -> Option<&DownloadTask> {
        self.tasks
            .iter()
            .find(|task| task.status == DownloadStatus::Queued)
    }
}
