# Contributing

## 分支策略

- 禁止直接在 `master/main` 上开发。
- 所有改动必须通过功能分支提交。
- 合并前必须发起 PR 并完成自测。

## 推荐流程

1. 新建分支（自动基于默认分支拉最新）：

```bash
./scripts/git_feature_flow.sh start feat/your-change
```

2. 开发并本地验证：

```bash
cargo test
```

3. 推送分支：

```bash
./scripts/git_feature_flow.sh push
```

4. 创建 PR：

```bash
gh pr create --fill
```

5. 通过 review 后再 merge。

## 提交建议

- 提交信息建议使用动词开头并描述影响面。
- 单个 PR 聚焦一个主题，避免混入无关改动。
- 如果涉及功能变化，请同步更新 `README.md`。
