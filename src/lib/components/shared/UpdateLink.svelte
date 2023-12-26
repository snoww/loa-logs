<script>
    export let href = "";
    export let title = undefined;
</script>

<a {href} {title} target="_blank">
    <slot></slot>
</a>