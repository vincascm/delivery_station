## deployment task

**repository:** {{repository_name}}

{% if repository_description -%}
**description:** {{repository_description}}
{%- endif %}

**status:** {% if status %} success {% else %} failure {% endif %}

**logs:**

{% for log in logs -%}
{{loop.index}}. {{log.description}} {% if log.stdout -%}
        [stdout]({{log.stdout}})
    {%- else -%}
        stdout
    {%- endif %}, {% if log.stderr -%}
        [stderr]({{log.stderr}})
    {%- else -%}
        stderr
    {%- endif %}
{% endfor %}
