# Screenshots

<div align=center style="display: flex; flex-wrap: wrap; justify-content: center; gap: 1rem;">

{% for screenshot in site.static_files %}
{% if screenshot.path contains "assets/screenshots" %}
<div style="display: flex; flex-direction: column; align-items: center; gap: 0.5rem;">
<a href="{{ screenshot.path }}">
<img src="{{ screenshot.path }}" alt="{{ screenshot.name }}" width=200>
<br/><small>{{ screenshot.name | replace: '_', ' ' | replace: '.png', '' | capitalize }}</small>
</a>
</div>
{% endif %}
{% endfor %}
</div>
