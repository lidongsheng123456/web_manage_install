//! 并发安装任务构建与执行。
//!
//! 将"构建任务列表"与"并发执行"拆到独立模块，
//! 让 orchestrator 只做流程编排，不承载并发细节。

use crate::common::types::{DownloadProgress, InstallConfig, InstallResult};
use crate::common::version_policy::defaults;
use crate::install::components::{bundled, jdk, maven, mysql, node, tomcat};
use crate::install::workflow::cancel::check_cancel;
use crate::install::workflow::dry_run::dry_run_download;
use crate::install::workflow::events::emit_status;
use crate::install::workflow::result::record_install_result;
use std::future::Future;
use std::pin::Pin;
use tauri::ipc::Channel;
use tauri::AppHandle;

type BoxFuture = Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;

struct ComponentTask {
    name: &'static str,
    future: BoxFuture,
}

/// 根据用户配置构建需要执行的组件任务列表，然后并发执行所有任务。
pub async fn run_all_concurrent(
    app: &AppHandle,
    config: &InstallConfig,
    on_progress: &Channel<DownloadProgress>,
) -> Vec<InstallResult> {
    let tasks = build_task_list(app, config, on_progress);
    if tasks.is_empty() {
        return Vec::new();
    }
    execute_concurrent(app, tasks).await
}

fn build_task_list(
    app: &AppHandle,
    config: &InstallConfig,
    on_progress: &Channel<DownloadProgress>,
) -> Vec<ComponentTask> {
    let root = config.install_root.clone();
    let temp = format!("{root}\\_temp");
    let dry = config.dry_run;
    let mut tasks = Vec::new();

    if config.install_nodejs {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        let ver = config.node_version.clone();
        tasks.push(ComponentTask {
            name: "nodejs",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "nodejs", &ver, &t, &p).await }
                else { node::install(&a, &r, &t, &ver, &p).await }
            }),
        });
    }

    if config.install_jdk {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        let ver = config.jdk_version.clone();
        tasks.push(ComponentTask {
            name: "jdk",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "jdk", &ver, &t, &p).await }
                else { jdk::install(&a, &r, &t, &ver, &p).await }
            }),
        });
    }

    if config.install_maven {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        let ver = config.maven_version.clone();
        tasks.push(ComponentTask {
            name: "maven",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "maven", &ver, &t, &p).await }
                else { maven::install(&a, &r, &t, &ver, &p).await }
            }),
        });
    }

    if config.install_mysql {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        let ver = config.mysql_version.clone();
        let pwd = config.mysql_password.clone();
        tasks.push(ComponentTask {
            name: "mysql",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "mysql", &ver, &t, &p).await }
                else { mysql::install(&a, &r, &t, &ver, &pwd, &p).await }
            }),
        });
    }

    if config.install_tomcat {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        let ver = config.tomcat_version.clone();
        tasks.push(ComponentTask {
            name: "tomcat",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "tomcat", &ver, &t, &p).await }
                else { tomcat::install(&a, &r, &t, &ver, &p).await }
            }),
        });
    }

    if config.install_idea {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        tasks.push(ComponentTask {
            name: "idea",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "idea", defaults::IDEA, &t, &p).await }
                else { bundled::download_idea(&a, &r, &t, &p).await }
            }),
        });
    }

    if config.install_navicat {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        tasks.push(ComponentTask {
            name: "navicat",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "navicat", defaults::NAVICAT, &t, &p).await }
                else { bundled::download_navicat(&a, &r, &t, &p).await }
            }),
        });
    }

    if config.install_redis {
        let (a, r, t, p) = (app.clone(), root.clone(), temp.clone(), on_progress.clone());
        tasks.push(ComponentTask {
            name: "redis",
            future: Box::pin(async move {
                if dry { dry_run_download(&a, "redis", defaults::REDIS, &t, &p).await }
                else { bundled::download_redis(&a, &r, &t, &p).await }
            }),
        });
    }

    tasks
}

async fn execute_concurrent(app: &AppHandle, tasks: Vec<ComponentTask>) -> Vec<InstallResult> {
    let mut handles = Vec::with_capacity(tasks.len());

    for task in tasks {
        let app_clone = app.clone();
        let name = task.name;
        handles.push((
            name,
            tokio::spawn(async move {
                if check_cancel(&app_clone).is_err() {
                    emit_status(&app_clone, name, "error", "安装已取消");
                    return (name, Err("安装已取消".to_string()));
                }
                (name, (task.future).await)
            }),
        ));
    }

    let mut results = Vec::with_capacity(handles.len());
    for (name, handle) in handles {
        match handle.await {
            Ok((_, result)) => {
                record_install_result(app, name, result, &mut results);
            }
            Err(e) => {
                record_install_result(
                    app,
                    name,
                    Err(format!("任务执行异常: {e}")),
                    &mut results,
                );
            }
        }
    }

    results
}
