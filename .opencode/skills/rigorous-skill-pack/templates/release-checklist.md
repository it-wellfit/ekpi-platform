# 發布檢查表 (Release Checklist)

**版本**: v1.0.0  
**日期**:  
**擁有者**:  

---

## 1. 建置與測試

- [ ] `cargo fmt -- --check` 通過
- [ ] `cargo clippy --all-targets -- -D warnings` 零警告
- [ ] `cargo test --locked` 全部通過

## 2. 部署

- [ ] 發布檢查表已執行
- [ ] 回滾程序已驗證
- [ ] 備份還原已測試

## 3. 可觀測性

- [ ] 監控已啟用
- [ ] SLO 已定義

## 4. 定義完成 (DoD)

- [ ] 所有檢查項通過
- [ ] 部署記錄已保存
