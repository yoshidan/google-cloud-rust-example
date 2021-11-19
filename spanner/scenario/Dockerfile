# Start with a base Python 3.7.2 image
FROM python:3.8

# Add the external tasks directory into /tasks
ADD locust-tasks /locust-tasks

# Install the required dependencies via pip
RUN pip install -r /locust-tasks/requirements.txt

# Set script to be executable
RUN chmod 755 /locust-tasks/run.sh

# Start Locust using LOCUS_OPTS environment variable
ENTRYPOINT ["/locust-tasks/run.sh"]