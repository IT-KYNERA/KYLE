# Animaciones y Transiciones

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [style-system.md](style-system.md) — Sistema de estilos
- [state-events.md](state-events.md) — Estado y eventos

---

## 1. Tipos de Animación

### 1.1 Transiciones

Cambio suave entre dos estados de estilo:

```kyx
<style<button> Primary:
    background = Color("#0066FF")
    transition = Transition("background", 200, Easing.EaseInOut, 0)

<style<button> PrimaryHover: Primary:
    background = Color("#0052CC")
    # La transición de 200ms se aplica automáticamente
```

### 1.2 Animaciones

Secuencia de fotogramas:

```kyle
animation FadeIn:
    from: opacity = 0.0
    to:   opacity = 1.0
    duration = 300
    easing = Easing.EaseOut

animation SlideIn:
    from:
        translate_x = -100
        opacity = 0.0
    to:
        translate_x = 0
        opacity = 1.0
    duration = 400
    easing = Easing.EaseOutCubic

animation Pulse:
    # Keyframes múltiples
    0%:   scale = 1.0
    50%:  scale = 1.05
    100%: scale = 1.0
    duration = 1000
    easing = Easing.EaseInOut
    iterations = Infinite
```

### 1.3 Micro-interacciones

Para feedback visual inmediato:

```kyle
animation Ripple:
    type = RippleEffect      # onda expansiva desde el punto de click
    duration = 600
    color = Color("#FFFFFF")
    max_opacity = 0.3

animation ScalePress:
    type = ScaleEffect
    pressed_scale = 0.95
    duration = 100
    easing = Easing.EaseOut
```

---

## 2. Uso en Componentes

### 2.1 Animación en estilo

```kyx
<card tpl=Elevated animation=FadeIn>
    <text value="Contenido" />
</card>
```

### 2.2 Animación condicional

```kyx
@(
    visible: ^bool = false
)
<view>
    <button text="Toggle" click=@visible = !visible />
    <dialog
        animation=if visible: FadeIn else: FadeOut
        visible=@visible
    >
        <text value="Contenido del diálogo" />
    </dialog>
</view>
```

### 2.3 Animación de lista

```kyx
<list items=@items animation=StaggeredList>
    # Cada item aparece con un delay incremental
</list>
```

### 2.4 Page transitions

```kyle
animation PageSlide:
    from:
        translate_x = 100   # viene desde la derecha
        opacity = 0.0
    to:
        translate_x = 0
        opacity = 1.0
    duration = 300
    easing = Easing.EaseOut
```

---

## 3. Easing Functions

```kyle
enum Easing:
    Linear
    Ease
    EaseIn
    EaseOut
    EaseInOut
    EaseInSine
    EaseOutSine
    EaseInOutSine
    EaseInCubic
    EaseOutCubic
    EaseInOutCubic
    EaseInBack
    EaseOutBack
    EaseInOutBack
    Elastic(amplitude: f32, period: f32)
    Bounce
    CubicBezier(x1: f32, y1: f32, x2: f32, y2: f32)
```

---

## 4. Compilación por Target

### 4.1 Web

Las animaciones Kyle se compilan a CSS animations/transitions + Web Animations API:

```javascript
// FadeIn → CSS
const style = document.createElement('style');
style.textContent = `
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    .anim-fade-in {
        animation: fadeIn 300ms ease-out;
    }
`;

// O usando Web Animations API para más control:
element.animate([
    { opacity: 0, transform: 'translateX(-100px)' },
    { opacity: 1, transform: 'translateX(0)' }
], {
    duration: 400,
    easing: 'cubic-bezier(0.22, 1, 0.36, 1)',
    fill: 'forwards'
});
```

### 4.2 Desktop (Skia)

Las animaciones se ejecutan en el loop de render:

```kyle
# Generado automáticamente
fn animate_fade_in(el, start_time: i64):
    elapsed = now() - start_time
    t = min(elapsed / 300.0, 1.0)
    eased = ease_out(t)
    el.opacity = eased
    if t < 1.0:
        request_frame(fn(): animate_fade_in(el, start_time))
```

---

## 5. Rendimiento

| Técnica | Descripción |
|---------|-------------|
| **GPU acelarada** | transform y opacity usan GPU en web (composite layers) |
| **Off-screen** | Animaciones fuera de pantalla no gastan recursos |
| **Throttle** | Máximo 60fps, menor en background |
| **Cancelación** | Al desmontar componente, se cancelan sus animaciones |
| **Reduced motion** | Respetar preferencia `prefers-reduced-motion` |

---

## 6. Referencias

- [style-system.md](style-system.md) — Sistema de estilos
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Page transitions
