## deployment task

**repository:** {{repository_name}}

**status:** {% if status %} success {% else %} failure {% endif %}

**logs:**

{% for log in logs -%}
{{loop.index}}. {% if log.0 | length() > 0 -%}
        [stdout]({{log.0}})
    {%- else -%}
        stdout
    {%- endif %}, {% if log.1 | length() > 0 -%}
        [stderr]({{log.1}})
    {%- else -%}
        stderr
    {%- endif %}
{% endfor %}
