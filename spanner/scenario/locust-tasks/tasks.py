#!/usr/bin/env python

# Copyright 2015 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

from locust import HttpUser, SequentialTaskSet, task, between, constant

class UserBehavior(SequentialTaskSet):
    user_id = ""
    def on_start(self):
        response = self.client.post('/CreateUser')
        self.user_id = response.text
        print(self.user_id)

    @task(1)
    def read_inventory(self):
        response = self.client.post(url='/ReadOnly', data={'user_id': self.user_id })

    @task(1)
    def update_inventory(self):
        response = self.client.post(url='/ReadWrite', data={'user_id': self.user_id })

class WebsiteUser(HttpUser):
    tasks = {UserBehavior:1}
    wait_time = constant(0.2)