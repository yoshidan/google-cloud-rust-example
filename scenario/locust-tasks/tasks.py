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


import uuid

from datetime import datetime
from locust import HttpUser, SequentialTaskSet, task, between, constant

class UserBehavior(SequentialTaskSet):
    token = ""
    gameDay = ""
    itemId = ""
    def on_start(self):
        response = self.client.post('/api/v1/auth/Register')
        self.token = (response.json()["token"])
        response2 = self.client.post(url='/api/v1/ranking/Add', headers={'x-auth-token': self.token})

    @task(1)
    def ranking_count(self):
        response = self.client.post(
            url='/api/v1/ranking/Count',
            headers={'x-auth-token': self.token })

    @task(1)
    def ranking_near_me(self):
        response = self.client.post(
            url='/api/v1/ranking/NearMe',
            headers={'x-auth-token': self.token})

    @task(1)
    def ranking_range(self):
        response = self.client.post(
            url='/api/v1/ranking/Range',
            headers={'x-auth-token': self.token})

class WebsiteUser(HttpUser):
    tasks = {UserBehavior:1}
    wait_time = constant(0.2)