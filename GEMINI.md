# Regras de Projeto Antigravity - PhotoCraft Android

Este projeto segue as diretrizes obrigatórias de engenharia, arquitetura em Rust e versionamento semântico estabelecidas em:
- [`.agents/rules/universal-engineering-rules.md`](.agents/rules/universal-engineering-rules.md)
- [`AGENT_INSTRUCTIONS.md`](AGENT_INSTRUCTIONS.md)
- [`VERSIONING.md`](VERSIONING.md)
- [`CHANGELOG.md`](CHANGELOG.md)

### Diretrizes de Ação Imediata para o Agente:
1. **Preservação Mobile**: Nunca remova ou reescreva a pasta `apps/photocraft-android/` sem necessidade explícita.
2. **Keystore Intocável**: Jamais apague ou regere `apps/photocraft-android/release.keystore`.
3. **Rust 2024 & Never Crash**: Proibido `unwrap()`, `expect()`, `panic!()` ou `todo!()` fora de testes.
4. **SemVer & Monotonia**: Incrementar a versão semântica e `version_code` a cada nova build.
5. **Documentação Contínua**: Registrar todo bug em `CHANGELOG.md` com padrão `[BUG-XXX]` e `[FIX-XXX]`.
