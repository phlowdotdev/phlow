# Implementação Responsiva - Botão "Run now in codespace"

## Funcionalidade Implementada

O botão "Run now in codespace" agora possui comportamento responsivo:

### Desktop (≥ 997px)
- ✅ Botão aparece no **Hero** da página inicial
- ✅ Botão **desaparece** do menu de navegação (navbar)

### Mobile (≤ 996px)
- ✅ Botão **desaparece** do Hero da página inicial  
- ✅ Botão aparece no **menu de navegação** (navbar)

## Arquivos Modificados

### 1. `docusaurus.config.ts`
```typescript
// Adicionado no array navbar.items:
{
  href: 'https://github.com/codespaces/new?repo=phlowdotdev/phlow-mirror-request',
  label: 'Run now in codespace',
  position: 'right',
  className: 'navbar-codespace-link', // Classe para controle de visibilidade
},
```

### 2. `src/pages/index.tsx`
```typescript
// Adicionada classe CSS hero-codespace-button:
<HomeButton
  className="button button--secondary button--lg button--start-codespace hero-codespace-button"
  target='_blank'
  to="https://github.com/codespaces/new?repo=phlowdotdev/phlow-mirror-request">
  Run now in codespace <CodespaceSvg />
</HomeButton>
```

### 3. `src/css/custom.css`
```css
/* No desktop: botão aparece no Hero, some do navbar */
@media (min-width: 997px) {
  .navbar-codespace-link {
    display: none !important;
  }
  
  .hero-codespace-button {
    display: flex !important;
  }
}

/* No mobile: botão some do Hero, aparece no navbar */
@media (max-width: 996px) {
  .navbar-codespace-link {
    display: block !important;
  }
  
  .hero-codespace-button {
    display: none !important;
  }
}
```

## Como Testar

1. **Desktop**: Acesse em uma tela ≥ 997px de largura
   - Verifique que o botão aparece no Hero
   - Verifique que o botão NÃO aparece no navbar

2. **Mobile**: Redimensione a janela para ≤ 996px ou acesse via mobile
   - Verifique que o botão NÃO aparece no Hero
   - Verifique que o botão aparece no navbar (menu hambúrguer no mobile)

## Breakpoint Escolhido

O breakpoint de `996px` foi escolhido para coincidir com o breakpoint padrão do Docusaurus para telas pequenas, garantindo consistência com o comportamento responsivo existente do site.

## URL de Teste

Servidor local: http://localhost:3001/

## Status

✅ Implementação completa
✅ Build bem-sucedido  
✅ Servidor de desenvolvimento rodando
✅ Pronto para testes
